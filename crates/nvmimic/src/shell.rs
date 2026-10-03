//! The calling shell: what nvm can see of it, and the changes nvm makes to it.
//!
//! nvm runs inside the user's shell, so it reads unexported variables and
//! leaves variables behind. A child process cannot do either, so the shell
//! side hands its variables in and evaluates the [`Effect`]s that come back.

use std::collections::{BTreeMap, BTreeSet};

/// Which shell nvm is running in, as far as nvm's behavior depends on it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dialect {
    Bash,
    /// bash in POSIX mode, which is what macOS's `/bin/sh` is.
    BashPosix,
    Zsh,
    /// dash or another plain POSIX shell.
    Posix,
}

impl Dialect {
    pub fn is_zsh(self) -> bool {
        self == Dialect::Zsh
    }

    pub fn is_bash(self) -> bool {
        matches!(self, Dialect::Bash | Dialect::BashPosix)
    }
}

/// One change to the calling shell, applied in order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Effect {
    Set {
        name: String,
        value: String,
        export: bool,
    },
    Export(String),
    Unset(String),
    UnsetFunctions(Vec<String>),
    /// nvm's `nvm_hash_reset`, run by the shell so it sees the shell's options.
    HashReset,
}

/// The variables nvm can see, which of them are exported, and the changes
/// made to them so far.
#[derive(Clone, Debug, Default)]
pub struct Vars {
    values: BTreeMap<String, String>,
    exported: BTreeSet<String>,
    effects: Vec<Effect>,
}

impl Vars {
    pub fn new(values: BTreeMap<String, String>, exported: BTreeSet<String>) -> Vars {
        Vars {
            values,
            exported,
            effects: Vec::new(),
        }
    }

    /// Variables that are all exported, as a child process sees them.
    pub fn from_env(values: BTreeMap<String, String>) -> Vars {
        let exported = values.keys().cloned().collect();
        Vars::new(values, exported)
    }

    /// `${NAME-}`: the value, or empty when unset.
    pub fn get(&self, name: &str) -> &str {
        self.values.get(name).map_or("", String::as_str)
    }

    /// `${NAME+x}`: whether the variable is set, even to an empty value.
    pub fn is_set(&self, name: &str) -> bool {
        self.values.contains_key(name)
    }

    pub fn get_opt(&self, name: &str) -> Option<&str> {
        self.values.get(name).map(String::as_str)
    }

    /// Names of exported variables, as awk's `ENVIRON` lists them.
    pub fn exported_names(&self) -> Vec<String> {
        self.exported
            .iter()
            .filter(|name| self.values.contains_key(*name))
            .cloned()
            .collect()
    }

    /// The environment a program started from this shell would get.
    pub fn exported(&self) -> Vec<(String, String)> {
        self.exported_names()
            .into_iter()
            .map(|name| {
                let value = self.values[&name].clone();
                (name, value)
            })
            .collect()
    }

    /// Every variable the shell would hand to nvm, exported or not.
    pub fn all(&self) -> Vec<(String, String)> {
        self.values
            .iter()
            .map(|(name, value)| (name.clone(), value.clone()))
            .collect()
    }

    /// `export NAME=value`.
    pub fn export(&mut self, name: &str, value: impl Into<String>) {
        let value = value.into();
        self.values.insert(name.to_string(), value.clone());
        self.exported.insert(name.to_string());
        self.effects.push(Effect::Set {
            name: name.to_string(),
            value,
            export: true,
        });
    }

    /// `NAME=value`, which keeps the variable exported if it already was.
    /// This is also how nvm leaks variables.
    pub fn assign(&mut self, name: &str, value: impl Into<String>) {
        let value = value.into();
        self.values.insert(name.to_string(), value.clone());
        self.effects.push(Effect::Set {
            name: name.to_string(),
            value,
            export: false,
        });
    }

    /// `export NAME` on a variable already holding its value.
    pub fn export_existing(&mut self, name: &str) {
        self.exported.insert(name.to_string());
        self.effects.push(Effect::Export(name.to_string()));
    }

    pub fn unset(&mut self, name: &str) {
        self.values.remove(name);
        self.exported.remove(name);
        self.effects.push(Effect::Unset(name.to_string()));
    }

    /// Sets a variable for the rest of this call without changing the shell,
    /// as `NAME=value func` and `local` do.
    pub fn set_quiet(&mut self, name: &str, value: &str) {
        self.values.insert(name.to_string(), value.to_string());
    }

    pub fn unset_quiet(&mut self, name: &str) {
        self.values.remove(name);
    }

    pub fn snapshot(&self) -> Vars {
        Vars::new(self.values.clone(), self.exported.clone())
    }

    pub fn push(&mut self, effect: Effect) {
        self.effects.push(effect);
    }

    pub fn effects(&self) -> &[Effect] {
        &self.effects
    }

    /// Forgets assignments to `name` made since `mark`, because they landed
    /// in a `local` that has gone out of scope.
    pub fn forget_since(&mut self, mark: usize, name: &str) {
        let mut index = 0;
        self.effects.retain(|effect| {
            let keep =
                index < mark || !matches!(effect, Effect::Set { name: set, .. } if set == name);
            index += 1;
            keep
        });
    }

    pub fn mark(&self) -> usize {
        self.effects.len()
    }

    /// Runs `body` on a copy, the way a `$( ... )` subshell runs: changes to
    /// variables made inside are thrown away.
    pub fn subshell<T>(&self, body: impl FnOnce(&mut Vars) -> T) -> T {
        let mut copy = self.snapshot();
        body(&mut copy)
    }
}

/// Quotes a value for any POSIX shell: single quotes, with each embedded
/// single quote written as `'\''`.
pub fn quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', r"'\''"))
}

/// Writes effects as shell code for the calling shell to `eval`.
pub fn render(effects: &[Effect]) -> String {
    let mut code = String::new();
    for effect in effects {
        match effect {
            Effect::Set {
                name,
                value,
                export: true,
            } => code.push_str(&format!("export {name}={}\n", quote(value))),
            Effect::Set {
                name,
                value,
                export: false,
            } => code.push_str(&format!("{name}={}\n", quote(value))),
            Effect::Export(name) => code.push_str(&format!("export {name}\n")),
            Effect::Unset(name) => code.push_str(&format!("unset {name}\n")),
            Effect::UnsetFunctions(names) => {
                code.push_str(&format!("unset -f {} >/dev/null 2>&1\n", names.join(" ")))
            }
            Effect::HashReset => code.push_str("nvm_hash_reset\n"),
        }
    }
    code
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quoting_survives_single_quotes() {
        assert_eq!(quote("it's"), r"'it'\''s'");
    }

    #[test]
    fn subshell_changes_do_not_escape() {
        let mut vars = Vars::default();
        vars.export("A", "1");
        vars.subshell(|inner| inner.export("A", "2"));
        assert_eq!(vars.get("A"), "1");
        assert_eq!(vars.effects().len(), 1);
    }
}
