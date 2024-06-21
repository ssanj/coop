# Coop

Visualise progress on your network file copy

![Coop](coop.gif)

## Usage

View usage with `coop -h`:

```
Making progress on your network file copy

Usage: coop [OPTIONS] --source <SOURCE> --destination-dir <DESTINATION_DIR>

Options:
      --verbose
          Verbose debug logging
  -s, --source <SOURCE>
          Source directory or file to copy files from
  -d, --destination-dir <DESTINATION_DIR>
          Destination directory to copy files to
  -c, --concurrency <CONCURRENCY>
          The maximum number of file copies to perform concurrently (1-16) [default: 4]
  -b, --buffer-size <BUFFER_SIZE>
          The maximum buffer size to use when copying. Maximum of 1024KB or 128MB. [default: 1MB]
  -i, --ignore <IGNORE>
          Files to ignore during copy [default: .DS_Store .git /target]
      --skip-verify
          Skip asking verification on copy
  -h, --help
          Print help (see more with '--help')
  -V, --version
          Print version
```

To copy all the files in a source directory to a destination directory:

```
coop -s <SOURCE_DIR> -d <DESTINATION_DIR>
```

This uses a default concurrency of 4 files and a buffer size of 1MB. `.git` and `.DS_Store` files are excluded by default.

## Installation

### Downloading a Release

Download the latest [release](https://github.com/ssanj/coop/releases) for your operating system (Linux or macOSX).
Make it executable with:

`chmod +x <COOP_EXEC>`

Copy executable to a directory on your path.


### Building through Cargo

You can build Coop through Cargo with:

```
cargo install --git https://github.com/ssanj/coop
```

This will install Coop into your Cargo home directory; usually `~/.cargo/bin`.

### Building from Source

Ensure you have Cargo installed.

Run:

```
cargo build --release
Copy binary file from target/release/coop to a directory on your PATH.
```
