use std::fs::File;
use std::io::{self, Write};
use std::process::Command;
use std::{env, fs};

const TEMPFILE: &str = ".vorbis_comment_set_metadata.tmp";

fn main() {
    let args: Vec<String> = env::args().collect();
    let non_progname_args = args.get(1..);

    if let Some(other_args) = non_progname_args {
        for filename in other_args {
            vorbis_edit_file(filename);
        }
    }
}

fn vorbis_edit_file(filename: &str) {
    let mut file_name = filename.to_owned();
    let (fname, ext) = file_name
        .split_once(".")
        .expect("vorbis-edit called with file that has no extension");
    assert_eq!(ext, "ogg");

    let _ = File::open(&file_name).expect("vorbis-edit called with nonexistent file");

    let (purported_filename, vorbis) = if let Some((artist, title)) = fname.split_once(" - ") {
        (fname, Some((artist.to_owned(), title.to_owned())))
    } else {
        (fname, None)
    };

    let (mut artist, mut title) = vorbis.unwrap_or(("unknown".to_owned(), "unknown".to_owned()));
    print!(
        "Would you like to change the filename?\nCurrent filename:{}\nCurrent artist: {}\nCurrent title: {}\n[y/N]: ",
        purported_filename, artist, title
    );
    io::stdout().flush().unwrap();
    let mut should_edit = String::new();
    io::stdin().read_line(&mut should_edit).unwrap_or_else(|_| {
        should_edit = String::from("");
        0
    });

    let rel_bytes = &should_edit.as_bytes()[0..3.min(should_edit.len())];
    let should_edit = match rel_bytes {
        b"yes" => true,
        b"ye" => true,
        b"y" => true,
        _ => false,
    };

    if should_edit {
        let mut n_artist = String::new();
        let mut n_title = String::new();
        print!("Enter artist: ");
        io::stdout().flush().unwrap();
        io::stdin()
            .read_line(&mut n_artist)
            .expect("did not give an artist");

        print!("Enter title: ");
        io::stdout().flush().unwrap();
        io::stdin()
            .read_line(&mut n_title)
            .expect("did not give a title");

        artist = n_artist.trim_end().to_owned().clone();
        title = n_title.trim_end().to_owned().clone();

        let new_filename = format!("{artist} - {title}.ogg");
        fs::rename(&file_name, &new_filename).expect("failed to rename file");
        println!("Renamed '{}' to '{}'", &file_name, &new_filename);

        file_name = new_filename.clone();
    }

    let _ = File::open(&file_name).expect("could not open file to verify it is real");
    // Just make sure the tempfile is clear
    Command::new("rm").arg("-f").arg(TEMPFILE).output().unwrap();
    File::create(TEMPFILE).unwrap();

    let boilerplate = format!(
        "{}\nartist={}\ntitle={}\n{}",
        "language=eng\nencoder=Lavc60.3.100 libvorbis", artist, title, "genre=\nalbum=\ndate=\n"
    );

    fs::write(TEMPFILE, boilerplate).unwrap();

    let mut nvim = Command::new("sh");
    nvim.arg("-c").arg(format!("nvim {TEMPFILE}").as_str());
    nvim.spawn().unwrap().wait().unwrap();

    let mut vorbiscomment = Command::new("vorbiscomment");
    vorbiscomment
        .arg("-c")
        .arg(TEMPFILE)
        .arg("-w")
        .arg(&file_name);
    vorbiscomment.output().unwrap();
    Command::new("rm").arg("-f").arg(TEMPFILE).output().unwrap();
}
