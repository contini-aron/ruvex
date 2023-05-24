use crate::errors::ConventionalCommitParseError;
use core::fmt;
use regex::Regex;
use ruvex_config::Config;

#[derive(Debug, PartialEq, Clone)]
pub struct ConventionalCommit {
    commit_type: String,       // feat
    short_sha: String,         //
    scope: Option<String>,     // ()
    breaking_change: bool,     // !
    short_description: String, // : to \n both excluded
    body: Option<String>,      // remainder of commit message
    footer: Option<String>,    // optional footer
}

// check for type in types list

impl fmt::Display for ConventionalCommit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut bc: &str = "";
        if self.breaking_change {
            bc = "!"
        }
        if let Some(ref bd) = self.body {
            write!(
                f,
                "{}{}: {}\n{}",
                self.commit_type, bc, self.short_description, bd
            )
        } else {
            write!(f, "{}{}: {}", self.commit_type, bc, self.short_description)
        }
    }
}

impl ConventionalCommit {
    #[allow(dead_code)]
    pub fn new(
        to_parse: &str,
        config: &Config,
        short_sha: String,
    ) -> Result<Self, ConventionalCommitParseError> {
        // check for :
        let mut cc_type: String;
        let mut scope: Option<String>;
        let mut breaking_change: bool = false;
        let regex: Regex = Regex::new(r"\(.+\)").unwrap();
        let short_description: String;
        let mut body: Option<String>;
        let mut footer: Option<String> = None;

        if !to_parse.contains(':') {
            return Err(ConventionalCommitParseError::MissingColumn);
        }

        if to_parse.contains("()") {
            return Err(ConventionalCommitParseError::EmptyScope);
        }

        let msg: String;
        //let (mut cc_type, msg) = to_parse.split_once(':').unwrap();
        (cc_type, msg) = match to_parse.split_once(": ") {
            Some(splitted) => {
                let (short, long) = splitted;
                (short.to_owned(), long.to_owned())
            }
            None => {
                return Err(ConventionalCommitParseError::NoSpaceAfterColumn);
            }
        };

        // check for optional scope
        scope = regex
            .captures(&cc_type)
            .map(|cap| cap.get(0).unwrap().as_str().to_owned());
        // if scope remove it from type and trim it
        if let Some(ref scope_str) = scope {
            cc_type = cc_type.replace(scope_str, "");
            scope = Some(scope_str.replace(['(', ')'], ""));
        }

        //check for ! and pop it
        if cc_type.ends_with('!') {
            breaking_change = true;
            cc_type = cc_type.replace('!', "");
        }

        // check if commit type is valid
        if !config.cc_type_in_config(&cc_type) {
            return Err(ConventionalCommitParseError::InvalidType {
                expected: config.cc_types.clone(),
                found: cc_type,
            });
        }

        (short_description, body) = match msg.split_once('\n') {
            Some(splitted) => {
                let (short, long) = splitted;
                if long.is_empty() {
                    (short.to_owned(), None)
                } else {
                    (short.to_owned(), Some(long.to_owned()))
                }
            }
            None => (msg.to_owned(), None),
        };

        // check for footer
        if let Some(ref text) = body {
            (body, footer) = match text.split_once("\n\n") {
                Some(splitted) => {
                    let (short, long) = splitted;
                    if long.is_empty() {
                        (Some(short.to_owned()), None)
                    } else {
                        (Some(short.to_owned()), Some(long.to_owned()))
                    }
                }
                None => (body, None),
            };
        }

        // check for BREAKING-CHANGE in footer
        if let Some(ref ft) = footer {
            if ft.contains("BREAKING-CHANGE: ") || ft.contains("BREAKING CHANGE: ") {
                breaking_change = true;
            }
        }
        Ok(ConventionalCommit {
            commit_type: cc_type,
            short_sha,
            scope,
            breaking_change,
            short_description,
            body,
            footer,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::errors::ConventionalCommitParseError;
    use crate::ConventionalCommit;
    use ruvex_config::Config;
    fn test_cc(cc_message: &str, cc_types: Vec<String>, sha: &str, cc_check: ConventionalCommit) {
        let cc = ConventionalCommit::new(cc_message, &Config { cc_types }, sha.to_owned()).unwrap();
        assert_eq!(cc, cc_check);
    }
    fn test_cc_error(
        cc_message: &str,
        cc_types: Vec<String>,
        sha: &str,
        error: ConventionalCommitParseError,
    ) {
        let cc = ConventionalCommit::new(cc_message, &Config { cc_types }, sha.to_owned());
        assert_eq!(cc.unwrap_err(), error);
    }
    #[test]
    fn classic_cc() {
        let cc_message = "feat(optional scope): good commit\nthis is a classic commit";
        let cc_types: Vec<String> = ["feat"].into_iter().map(String::from).collect();
        let sha = "ababa";
        let cc = ConventionalCommit {
            commit_type: "feat".to_owned(),
            short_sha: "ababa".to_owned(),
            scope: Some("optional scope".to_owned()),
            breaking_change: false,
            short_description: "good commit".to_owned(),
            body: Some("this is a classic commit".to_owned()),
            footer: None,
        };
        test_cc(cc_message, cc_types, sha, cc);
    }
    #[test]
    fn sad_brackets() {
        let cc_message = "feat(opti(onal scope): good commit\nthis is a classic commit";
        let cc_types: Vec<String> = ["feat"].into_iter().map(String::from).collect();
        let sha = "ababa";
        let cc = ConventionalCommit {
            commit_type: "feat".to_owned(),
            short_sha: "ababa".to_owned(),
            scope: Some("optional scope".to_owned()),
            breaking_change: false,
            short_description: "good commit".to_owned(),
            body: Some("this is a classic commit".to_owned()),
            footer: None,
        };
        test_cc(cc_message, cc_types, sha, cc);
        let cc_message = "feat(opti(onal sco)pe): good commit\nthis is a classic commit";
        let cc_types: Vec<String> = ["feat"].into_iter().map(String::from).collect();
        let sha = "ababa";
        let cc = ConventionalCommit {
            commit_type: "feat".to_owned(),
            short_sha: "ababa".to_owned(),
            scope: Some("optional scope".to_owned()),
            breaking_change: false,
            short_description: "good commit".to_owned(),
            body: Some("this is a classic commit".to_owned()),
            footer: None,
        };
        test_cc(cc_message, cc_types, sha, cc);
    }
    #[test]
    fn no_body() {
        let cc_message = "feat: good commit\n";
        let cc_types: Vec<String> = ["feat"].into_iter().map(String::from).collect();
        let sha = "ababa";
        let cc = ConventionalCommit {
            commit_type: "feat".to_owned(),
            short_sha: "ababa".to_owned(),
            scope: None,
            breaking_change: false,
            short_description: "good commit".to_owned(),
            body: None,
            footer: None,
        };
        test_cc(cc_message, cc_types, sha, cc);
    }
    #[test]
    fn breaking_change_symbol() {
        let cc_message = "feat!: good commit\n";
        let cc_types: Vec<String> = ["feat"].into_iter().map(String::from).collect();
        let sha = "ababa";
        let cc = ConventionalCommit {
            commit_type: "feat".to_owned(),
            short_sha: "ababa".to_owned(),
            scope: None,
            breaking_change: true,
            short_description: "good commit".to_owned(),
            body: None,
            footer: None,
        };
        test_cc(cc_message, cc_types, sha, cc);
    }
    #[test]
    fn breaking_change_body() {
        let cc_message = "feat: good commit\nbodyisbody\n\nBREAKING-CHANGE: sad change";
        let cc_types: Vec<String> = ["feat"].into_iter().map(String::from).collect();
        let sha = "ababa";
        let cc = ConventionalCommit {
            commit_type: "feat".to_owned(),
            short_sha: "ababa".to_owned(),
            scope: None,
            breaking_change: true,
            short_description: "good commit".to_owned(),
            body: Some("bodyisbody".to_owned()),
            footer: Some("BREAKING-CHANGE: sad change".to_owned()),
        };
        test_cc(cc_message, cc_types, sha, cc);
    }
    #[test]
    fn footer() {
        let cc_message = "feat: good commit\nbodyisbody\n\nchange: sad change";
        let cc_types: Vec<String> = ["feat"].into_iter().map(String::from).collect();
        let sha = "ababa";
        let cc = ConventionalCommit {
            commit_type: "feat".to_owned(),
            short_sha: "ababa".to_owned(),
            scope: None,
            breaking_change: false,
            short_description: "good commit".to_owned(),
            body: Some("bodyisbody".to_owned()),
            footer: Some("change: sad change".to_owned()),
        };
        test_cc(cc_message, cc_types, sha, cc);
    }
    #[test]
    fn missing_column() {
        let cc_message = "feat good commit\n";
        let cc_types: Vec<String> = ["feat"].into_iter().map(String::from).collect();
        let sha = "ababa";
        test_cc_error(
            cc_message,
            cc_types,
            sha,
            ConventionalCommitParseError::MissingColumn,
        );
    }
    #[test]
    fn invalid_type() {
        let cc_message = "fea: good commit\n";
        let cc_types: Vec<String> = ["feat"].into_iter().map(String::from).collect();
        let sha = "ababa";
        test_cc_error(
            cc_message,
            cc_types.clone(),
            sha,
            ConventionalCommitParseError::InvalidType {
                expected: cc_types,
                found: "fea".to_owned(),
            },
        );
    }
    #[test]
    fn empty_scope() {
        let cc_message = "feat(): good commit\n";
        let cc_types: Vec<String> = ["feat"].into_iter().map(String::from).collect();
        let sha = "ababa";
        test_cc_error(
            cc_message,
            cc_types.clone(),
            sha,
            ConventionalCommitParseError::EmptyScope,
        );
    }
}
