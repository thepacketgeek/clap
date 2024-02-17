use clap::Parser;

use crate::utils::assert_output;

#[test]
fn test_safely_nest_parser() {
    #[derive(Parser, Debug, PartialEq)]
    struct Opt {
        #[command(flatten)]
        foo: Foo,
    }

    #[derive(Parser, Debug, PartialEq)]
    struct Foo {
        #[arg(long)]
        foo: bool,
    }

    assert_eq!(
        Opt {
            foo: Foo { foo: true }
        },
        Opt::try_parse_from(["test", "--foo"]).unwrap()
    );
}

#[test]
fn implicit_struct_group() {
    #[derive(Parser, Debug)]
    struct Opt {
        #[arg(short, long, requires = "Source")]
        add: bool,

        #[command(flatten)]
        source: Source,
    }

    #[derive(clap::Args, Debug)]
    struct Source {
        crates: Vec<String>,
        #[arg(long)]
        path: Option<std::path::PathBuf>,
        #[arg(long)]
        git: Option<String>,
    }

    const OUTPUT: &str = "\
error: the following required arguments were not provided:
  <CRATES|--path <PATH>|--git <GIT>>

Usage: prog --add <CRATES|--path <PATH>|--git <GIT>>

For more information, try '--help'.
";
    assert_output::<Opt>("prog --add", OUTPUT, true);

    use clap::Args;
    assert_eq!(Source::group_id(), Some(clap::Id::from("Source")));
    assert_eq!(Opt::group_id(), Some(clap::Id::from("Opt")));
}

#[test]
fn skip_group_avoids_duplicate_ids() {
    #[derive(Parser, Debug)]
    #[group(skip)]
    struct Opt {
        #[command(flatten)]
        first: Compose<Empty, Empty>,
        #[command(flatten)]
        second: Compose<Empty, Empty>,
    }

    #[derive(clap::Args, Debug)]
    #[group(skip)]
    pub struct Compose<L: clap::Args, R: clap::Args> {
        #[command(flatten)]
        pub left: L,
        #[command(flatten)]
        pub right: R,
    }

    #[derive(clap::Args, Clone, Copy, Debug)]
    #[group(skip)]
    pub struct Empty;

    use clap::CommandFactory;
    Opt::command().debug_assert();

    use clap::Args;
    assert_eq!(Empty::group_id(), None);
    assert_eq!(Compose::<Empty, Empty>::group_id(), None);
    assert_eq!(Opt::group_id(), None);
}

#[test]
fn optional_flatten() {
    #[derive(Parser, Debug, PartialEq, Eq)]
    struct Opt {
        #[command(flatten)]
        source: Option<Source>,
    }

    #[derive(clap::Args, Debug, PartialEq, Eq)]
    struct Source {
        crates: Vec<String>,
        #[arg(long)]
        path: Option<std::path::PathBuf>,
        #[arg(long)]
        git: Option<String>,
    }

    assert_eq!(Opt { source: None }, Opt::try_parse_from(["test"]).unwrap());
    assert_eq!(
        Opt {
            source: Some(Source {
                crates: vec!["serde".to_owned()],
                path: None,
                git: None,
            }),
        },
        Opt::try_parse_from(["test", "serde"]).unwrap()
    );
    assert_eq!(
        Opt {
            source: Some(Source {
                crates: Vec::new(),
                path: Some("./".into()),
                git: None,
            }),
        },
        Opt::try_parse_from(["test", "--path=./"]).unwrap()
    );
}

#[test]
#[should_panic = "\
Command clap: Argument group name must be unique

\t'Compose' is already in use"]
fn helpful_panic_on_duplicate_groups() {
    #[derive(Parser, Debug)]
    struct Opt {
        #[command(flatten)]
        first: Compose<Empty, Empty>,
        #[command(flatten)]
        second: Compose<Empty, Empty>,
    }

    #[derive(clap::Args, Debug)]
    pub struct Compose<L: clap::Args, R: clap::Args> {
        #[command(flatten)]
        pub left: L,
        #[command(flatten)]
        pub right: R,
    }

    #[derive(clap::Args, Clone, Copy, Debug)]
    pub struct Empty;

    use clap::CommandFactory;
    Opt::command().debug_assert();
}

#[test]
fn custom_group_id() {
    #[derive(Parser, Debug, PartialEq, Eq)]
    struct Opt {
        #[command(flatten)]
        source: Option<Source>,
    }

    #[derive(clap::Args, Debug, PartialEq, Eq)]
    #[group(id = "source")]
    struct Source {
        crates: Vec<String>,
        #[arg(long)]
        path: Option<std::path::PathBuf>,
        #[arg(long)]
        git: Option<String>,
    }

    assert_eq!(Opt { source: None }, Opt::try_parse_from(["test"]).unwrap());
    assert_eq!(
        Opt {
            source: Some(Source {
                crates: vec!["serde".to_owned()],
                path: None,
                git: None,
            }),
        },
        Opt::try_parse_from(["test", "serde"]).unwrap()
    );
    assert_eq!(
        Opt {
            source: Some(Source {
                crates: Vec::new(),
                path: Some("./".into()),
                git: None,
            }),
        },
        Opt::try_parse_from(["test", "--path=./"]).unwrap()
    );
}

#[test]
fn required_group() {
    #[derive(Parser, Debug, PartialEq, Eq)]
    struct Opt {
        #[command(flatten)]
        source: Source,
    }

    #[derive(clap::Args, Debug, PartialEq, Eq)]
    #[group(required = true, multiple = false)]
    struct Source {
        #[arg(long)]
        path: Option<std::path::PathBuf>,
        #[arg(long)]
        git: Option<String>,
    }

    assert_eq!(
        Opt {
            source: Source {
                path: Some("./".into()),
                git: None,
            },
        },
        Opt::try_parse_from(["test", "--path=./"]).unwrap()
    );

    const OUTPUT: &str = "\
error: the following required arguments were not provided:
  <--path <PATH>|--git <GIT>>

Usage: test <--path <PATH>|--git <GIT>>

For more information, try '--help'.
";
    assert_output::<Opt>("test", OUTPUT, true);
}

#[test]
fn group_with_use_default() {
    #[derive(Parser, Debug, PartialEq, Eq)]
    struct Opt {
        #[command(flatten)]
        source: Source,
    }

    #[derive(clap::Args, Debug, PartialEq, Eq)]
    #[group(required = true, multiple = false, use_default = "other")]
    struct Source {
        #[arg(long)]
        one: bool,
        #[arg(long, default_value_t = true)]
        other: bool,
        #[arg(long)]
        something_else: bool,
    }

    assert_eq!(
        Opt {
            source: Source {
                one: false,
                other: true,
                something_else: false,
            },
        },
        Opt::try_parse_from(["test"]).unwrap()
    );
}
