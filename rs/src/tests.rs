use std::fs;
use duct::cmd;

#[test]
fn integration() {
    // Copy test data to a temporary location
    let d = tempfile::tempdir_in(".").expect("creating temp directory");
    let p = d.path().to_str().unwrap();
    fs::create_dir(format!("{}/a", p)).unwrap();
    fs::create_dir(format!("{}/a/b", p)).unwrap(); 
    fs::copy("./test_data/1.jpg", format!("{}/a/1.jpg", p)).unwrap();
    fs::copy("./test_data/2.jpg", format!("{}/a/2.jpg", p)).unwrap();
    fs::copy("./test_data/3.jpg", format!("{}/a/3.jpg", p)).unwrap();
    fs::copy("./test_data/4.jpg", format!("{}/a/b/4.jpg", p)).unwrap();

    cmd!("./target/debug/spg", "--config-path", ".spg", "init")
        .dir(&p).run().expect("spg init");

    cmd!("./target/debug/spg", "--config-path", ".spg", "add", "a/1.jpg")
        .dir(&p).run().expect("adding 1.jpg");

    cmd!("./target/debug/spg", "--config-path", ".spg", "add", "a/2.jpg")
        .dir(&p).run().expect("adding 2.jpg");

    assert_eq!(
        cmd!("./target/debug/spg", "--config-path", ".spg", "stat", "a/1.jpg")
            .dir(&p).read().expect("stat 1.jpg"),
        "The image is in the gallery.");

    assert_eq!(
        cmd!("./target/debug/spg", "--config-path", ".spg", "stat", "a/2.jpg")
            .dir(&p).read().expect("stat 2.jpg"),
        "The image is in the gallery.");

    cmd!("./target/debug/spg", "--config-path", ".spg", "rm", "a/2.jpg")
        .dir(&p).run().expect("removing 2.jpg");

    assert_eq!(
        cmd!("./target/debug/spg", "--config-path", ".spg", "stat", "a/1.jpg")
            .dir(&p).read().expect("stat 1.jpg"),
        "The image is in the gallery.");

    assert_eq!(
        cmd!("./target/debug/spg", "--config-path", ".spg", "stat", "a/2.jpg")
            .dir(&p).read().expect("stat 2.jpg"),
        "Nothing is in the gallery with this path.");

    fs::remove_file(format!("{}/a/2.jpg", p)).unwrap();

    cmd!("./target/debug/spg", "--config-path", ".spg", "sync", "a")
        .dir(&p).run().expect("sync a/");

    assert_eq!(
        cmd!("./target/debug/spg", "--config-path", ".spg", "stat", "a/1.jpg")
            .dir(&p).read().expect("stat 1.jpg"),
        "The image is in the gallery.");

    assert_eq!(
        cmd!("./target/debug/spg", "--config-path", ".spg", "stat", "a/3.jpg")
            .dir(&p).read().expect("stat 3.jpg"),
        "The image is in the gallery.");

    assert_eq!(
        cmd!("./target/debug/spg", "--config-path", ".spg", "stat", "a/b/4.jpg")
            .dir(&p).read().expect("stat 4.jpg"),
        "The image is in the gallery.");

    // TODO(arjun): It is possible to have an image in the gallery with a path that is no longer
    // valid. But, there is no way to test for this situation at the moment.
    // assert_eq!(
    //     cmd!("./target/debug/spg", "--config-path", ".spg", "stat", "a/2.jpg")
    //         .dir(&p).read().expect("stat 2.jpg"),
    //     "Nothing is in the gallery with this path.");
}