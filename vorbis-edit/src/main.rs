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
    assert!(file_name.len() > 4); // Must be more than ".ogg"
    let fname = (&file_name[0..file_name.len() - 4]).to_owned();
    let ext = &file_name[file_name.len() - 4..];
    assert_eq!(ext, ".ogg");

    let _ = File::open(&file_name).expect("vorbis-edit called with nonexistent file");

    let (purported_filename, vorbis) = if let Some((artist, title)) = fname.split_once(" - ") {
        (fname.clone(), Some((artist.to_owned(), title.to_owned())))
    } else {
        (fname.clone(), None)
    };

    let (mut artist, mut title) = vorbis.unwrap_or((fname.clone(), fname.clone()));
    print!(
        "Would you like to change the filename?\nCurrent filename: {}\nCurrent artist: {}\nCurrent title: {}\n[y/N]: ",
        purported_filename, artist, title
    );
    io::stdout().flush().unwrap();
    let mut should_edit = String::new();
    io::stdin().read_line(&mut should_edit).unwrap_or_else(|_| {
        should_edit = String::from("");
        0
    });

    let rel_bytes = should_edit.as_bytes()[0];
    let should_edit = match rel_bytes {
        b'y' => true,
        _ => false,
    };

    if should_edit {
        let mut n_artist = String::new();
        let mut n_title = String::new();

        get_user_input_or_default("Enter artist", &mut n_artist, &artist);
        get_user_input_or_default("Enter title", &mut n_title, &title);

        artist = n_artist.trim_end().to_owned().clone();
        title = n_title.trim_end().to_owned().clone();

        let new_filename = format!("{artist} - {title}.ogg");
        let Ok(()) = fs::rename(&file_name, &new_filename) else {
            panic!("failed to rename '{file_name}' to '{new_filename}'");
        };
        println!("Renamed '{}' to '{}'", &file_name, &new_filename);

        file_name = new_filename.clone();
    }

    // Just make sure any "mv" works
    let _ = File::open(&file_name).expect("could not open file to verify it is real");
    let boilerplate = format!(
        "{}\nartist={}\ntitle={}\n{}",
        "language=eng\nencoder=Lavc60.3.100 libvorbis", artist, title, "genre=\nalbum=\ndate=\n"
    );

    fs::write(TEMPFILE, boilerplate).unwrap();

    edit_vorbis_tempfile(TEMPFILE);
    set_vorbis_metadata(&file_name, TEMPFILE);
}

fn get_user_input_or_default(prompt: &str, place_into: &mut String, default: &str) {
    let printed = format!("{prompt} (default: '{default}'): ");
    print!("{}", printed);
    io::stdout().flush().unwrap();
    let mut buf = String::new();
    io::stdin().read_line(&mut buf).unwrap();
    let buf_nice = buf.trim_end();
    if buf_nice.len() == 0 {
        default.clone_into(place_into);
    } else {
        buf_nice.clone_into(place_into);
    }
}

fn edit_vorbis_tempfile(file: &str) {
    let mut sh = Command::new("sh");
    sh.arg("-c").arg(format!("nvim {file}").as_str());
    sh.spawn().unwrap().wait().unwrap();
}
fn set_vorbis_metadata(ogg_file: &str, tempfile: &str) {
    let mut vc = Command::new("vorbiscomment");
    vc.arg("-c").arg(tempfile).arg("-w").arg(ogg_file);
    vc.output().unwrap();
    // Clean up tempfile
    Command::new("rm").arg("-f").arg(tempfile).output().unwrap();
}
