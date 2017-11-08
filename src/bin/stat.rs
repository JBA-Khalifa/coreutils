#![deny(warnings)]

extern crate arg_parser;
extern crate extra;
extern crate time;
extern crate redox_users;

use std::{env, fmt, fs};
use std::io::{stdout, stderr, Write};
use arg_parser::ArgParser;
use extra::option::OptionalExt;
use redox_users::{get_group_by_id, get_user_by_id};
use std::os::unix::fs::MetadataExt;
use time::Timespec;

const MAN_PAGE: &'static str = /* @MANSTART{stat} */ r#"
NAME
    stat - display file status

SYNOPSIS
    stat [ -h | --help ] FILE...

DESCRIPTION
    Displays file status.

OPTIONS
    --help, -h
        print this message
"#; /* @MANEND */

const TIME_FMT: &'static str = "%Y-%m-%d %H:%M:%S.%f %z";

struct Perms(u32);

impl fmt::Display for Perms {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(0{:o}/", self.0 & 0o777)?;
        let perm = |i, c| {
            if self.0 & ((1 << i) as u32) != 0 {
                c
            } else {
                "-"
            }
        };
        write!(f, "{}{}{}", perm(8, "r"), perm(7, "w"), perm(6, "x"))?;
        write!(f, "{}{}{}", perm(5, "r"), perm(4, "w"), perm(3, "x"))?;
        write!(f, "{}{}{}", perm(2, "r"), perm(1, "w"), perm(0, "x"))?;
        write!(f, ")")?;
        Ok(())
    }
}

fn main() {
    let stdout = stdout();
    let mut stdout = stdout.lock();
    let mut stderr = stderr();
    let mut parser = ArgParser::new(1)
        .add_flag(&["h", "help"]);
    parser.parse(env::args());

    if parser.found("help") {
        stdout.write_all(MAN_PAGE.as_bytes()).try(&mut stderr);
        stdout.flush().try(&mut stderr);
        return;
    }

    for path in &parser.args[0..] {
        let meta = fs::symlink_metadata(path).unwrap();
        let file_type = if meta.file_type().is_symlink() {
            "symbolic link"
        } else if meta.is_file() {
            "regular file"
        } else if meta.is_dir() {
            "directory"
        } else {
            ""
        };
        if meta.file_type().is_symlink() {
            println!("File: {} -> {}", path, fs::read_link(path).unwrap().display());
        } else {
            println!("File: {}", path);
        }
        println!("Size: {}  Blocks: {}  IO Block: {} {}", meta.size(), meta.blocks(), meta.blksize(), file_type);
        println!("Device: {}  Inode: {}  Links: {}", meta.dev(), meta.ino(), meta.nlink());
        let user_option = get_user_by_id(meta.uid() as usize);
        let group_option = get_group_by_id(meta.gid() as usize);

        let username = user_option.map_or_else(|| String::from("UNKNOWN"), |user| user.user);
        let groupname = group_option.map_or_else(|| String::from("UNKNOWN"), |group| group.group);
        println!("Access: {}  Uid: ({}/{})  Gid: ({}/{})", Perms(meta.mode()),
                                                             meta.uid(), username,
                                                             meta.gid(), groupname);
        println!("Access: {}", time::at(Timespec::new(meta.atime(), meta.atime_nsec() as i32)).strftime(TIME_FMT).unwrap());
        println!("Modify: {}", time::at(Timespec::new(meta.mtime(), meta.mtime_nsec() as i32)).strftime(TIME_FMT).unwrap());
        println!("Change: {}", time::at(Timespec::new(meta.ctime(), meta.ctime_nsec() as i32)).strftime(TIME_FMT).unwrap());
    }
}
