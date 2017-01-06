extern crate tempdir;

use std::fs::File;

use super::super::{Exec, Redirection, NullFile, ExitStatus};

use self::tempdir::TempDir;

use tests::common::read_whole_file;

#[test]
fn exec_join() {
    let status = Exec::cmd("true").join().unwrap();
    assert_eq!(status, ExitStatus::Exited(0));
}

#[test]
fn null_file() {
    let mut p = Exec::cmd("cat")
        .stdin(NullFile).stdout(Redirection::Pipe)
        .popen().unwrap();
    let (out, _) = p.communicate(None).unwrap();
    assert!(out.unwrap() == "");
}

#[test]
fn stream_stdout() {
    let stream = Exec::cmd("echo")
        .args(&["-n", "foo"])
        .stream_stdout().unwrap();
    assert!(read_whole_file(stream) == "foo");
}

#[test]
fn stream_stderr() {
    let stream = Exec::cmd("sh")
        .args(&["-c", "echo -n foo >&2"])
        .stream_stderr().unwrap();
    assert!(read_whole_file(stream) == "foo");
}

#[test]
fn stream_stdin() {
    let tmpdir = TempDir::new("test").unwrap();
    let tmpname = tmpdir.path().join("output");
    {
        let mut stream = Exec::cmd("cat")
            .stdout(File::create(&tmpname).unwrap())
            .stream_stdin().unwrap();
        stream.write_all(b"foo").unwrap();
    }
    assert_eq!(read_whole_file(File::open(&tmpname).unwrap()), "foo");
}

#[test]
fn stream_capture_out() {
    let c = Exec::cmd("printf").arg("foo")
        .stdout(Redirection::Pipe).capture().unwrap();
    assert_eq!(c.stdout_str(), "foo");
}

#[test]
fn stream_capture_err() {
    let c = Exec::cmd("sh").arg("-c").arg("printf foo >&2")
        .stderr(Redirection::Pipe).capture().unwrap();
    assert_eq!(c.stderr_str(), "foo");
}

#[test]
fn stream_capture_out_with_input_data() {
    let c = Exec::cmd("cat")
        .stdin("foo")
        .stdout(Redirection::Pipe)
        .capture().unwrap();
    assert_eq!(c.stdout_str(), "foo");
}

#[test]
fn exec_shell() {
    // note: this uses built-in echo on Windows, so don't try anything
    // fancy like echo -n
    let stream = Exec::shell("echo foo").stream_stdout().unwrap();
    assert_eq!(read_whole_file(stream).trim(), "foo");
}

#[test]
fn pipeline_open() {
    let mut processes = {
        Exec::cmd("echo").arg("foo\nbar") | Exec::cmd("wc").arg("-l")
    }
    .stdout(Redirection::Pipe).popen().unwrap();
    let (output, _) = processes[1].communicate(None).unwrap();
    assert!(output.unwrap().trim() == "2");
}

#[test]
fn pipeline_stream_out() {
    let stream = {
        Exec::cmd("echo").arg("foo\nbar") | Exec::cmd("wc").arg("-l")
    }.stream_stdout().unwrap();
    assert!(read_whole_file(stream).trim() == "2");
}

#[test]
fn pipeline_stream_in() {
    let tmpdir = TempDir::new("test").unwrap();
    let tmpname = tmpdir.path().join("output");
    {
        let mut stream = {
            Exec::cmd("cat")
          | Exec::cmd("wc").arg("-l")
        }.stdout(File::create(&tmpname).unwrap())
         .stream_stdin().unwrap();
        stream.write_all(b"foo\nbar\nbaz\n").unwrap();
    }
    assert_eq!(read_whole_file(File::open(&tmpname).unwrap()).trim(), "3");
}

#[test]
fn pipeline_compose_pipelines() {
    let pipe1 = Exec::cmd("echo").arg("foo\nbar\nfoo") | Exec::cmd("sort");
    let pipe2 = Exec::cmd("uniq") | Exec::cmd("wc").arg("-l");
    let pipe = pipe1 | pipe2;
    let stream = pipe.stream_stdout().unwrap();
    assert_eq!(read_whole_file(stream).trim(), "2");
}

#[test]
fn pipeline_capture() {
    let c = {
        Exec::cmd("cat") | Exec::shell("wc -l")
    }.stdin("foo\nbar\nbaz\n").capture().unwrap();
    assert_eq!(c.stdout_str().trim(), "3");
}

#[test]
fn pipeline_join() {
    let status = (Exec::cmd("true") | Exec::cmd("true")).join().unwrap();
    assert_eq!(status, ExitStatus::Exited(0));

    let status = (Exec::cmd("false") | Exec::cmd("true")).join().unwrap();
    assert_eq!(status, ExitStatus::Exited(0));

    let status = (Exec::cmd("true") | Exec::cmd("false")).join().unwrap();
    assert_eq!(status, ExitStatus::Exited(1));
}

#[test]
fn pipeline_invalid_1() {
    let try = (Exec::cmd("echo").arg("foo") | Exec::cmd("no-such-command")).join();
    assert!(try.is_err());
}

#[test]
fn pipeline_invalid_2() {
    let try = (Exec::cmd("no-such-command") | Exec::cmd("echo").arg("foo")).join();
    assert!(try.is_err());
}