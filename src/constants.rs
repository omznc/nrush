// Version colors
pub const MAJOR: &str = "\x1b[31m";
pub const MINOR: &str = "\x1b[33m";
pub const PATCH: &str = "\x1b[32m";
pub const RESET: &str = "\x1b[0m";

// Other colors
pub const GRAY: &str = "\x1b[90m";

pub const HELP: &str = r"
USAGE:
    nrush [command] or [options] # Can't use both at the same time
COMMANDS:
    help        Prints help information
    source      Prints source code location
OPTIONS:
    -u, --update            Update all packages
    -i, --interactive       Interactive mode
    -p, --path              Path to package.json
    -s, --semver            Update up to the specified semver type (major, minor, patch)
    --include               Include dev and/or peer dependencies, eg. --include dev,peer
    --skip-ranges           Skip version ranges (e.g. ^, ~, >=, <=)
";

pub const ABOUT: &str = r"
AUTHOR:
    Omar Žunić <oss@omarzunic.com>
    https://omarzunic.com

SOURCE:
    https://github.com/omznc/nrush
";
