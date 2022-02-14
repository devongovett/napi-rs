use std::{
  env,
  fs::{self, File},
  io::{self, Write},
  path::Path,
};

pub fn write_file<P: AsRef<Path>>(path: &P, content: &str) -> Result<(), io::Error> {
  let path = path.as_ref();
  println!("Writing file: {}", path.display());
  if env::var("NAPI_DEBUG").is_ok() {
    println!("{}", &content);
  } else {
    let dir = path.parent().unwrap();
    fs::create_dir_all(dir)?;
    let mut file = File::create(path)?;
    file.write_all(content.as_bytes())?;
  }

  Ok(())
}
