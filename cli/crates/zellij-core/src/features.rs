//! A complete rendering selection, shared by native and hosted build paths.
//! Typed flags keep arbitrary CLI text out of generated container commands.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Features {
    pub maths: bool,
    pub diagrams: bool,
}

impl Features {
    pub fn as_str(self) -> &'static str {
        match (self.maths, self.diagrams) {
            (false, false) => "none",
            (true, false) => "maths",
            (false, true) => "diagrams",
            (true, true) => "maths,diagrams",
        }
    }
}

impl std::str::FromStr for Features {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "none" => Ok(Self::default()),
            "maths" => Ok(Self { maths: true, diagrams: false }),
            "diagrams" => Ok(Self { maths: false, diagrams: true }),
            "all" | "maths,diagrams" | "diagrams,maths" => Ok(Self { maths: true, diagrams: true }),
            _ => Err("choose maths, diagrams, maths,diagrams, all, or none; all/none must be used alone and duplicate features are not accepted".into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn complete_selections_have_canonical_shell_safe_spellings() {
        for (input, canonical) in [
            ("none", "none"),
            ("maths", "maths"),
            ("diagrams", "diagrams"),
            ("all", "maths,diagrams"),
            ("diagrams,maths", "maths,diagrams"),
        ] {
            assert_eq!(input.parse::<Features>().unwrap().as_str(), canonical);
        }
        for invalid in [
            "",
            "math",
            "all,maths",
            "none,diagrams",
            "maths,maths",
            "maths,",
            " maths",
            "maths\n",
            "$(touch file)",
            "maths\";exit 0",
        ] {
            assert!(invalid.parse::<Features>().is_err(), "{invalid:?}");
        }
    }
}
