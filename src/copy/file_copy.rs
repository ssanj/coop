use std::path::{Path, PathBuf};
use indicatif::MultiProgress;
use tokio::fs::{DirBuilder, File};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::monitor::MonitorMux;
use crate::progress::MyProgressBar;
use crate::model::{FileType, SizeComparison, R, CopyError};
use crate::args::BufferSize;
use super::SourceFile;

#[derive(Debug, Clone)]
pub struct FileCopy {
  source_file: SourceFile,
  destination_dir_path: PathBuf,
  progress_bar: MyProgressBar,
}

impl FileCopy {

  pub fn new<D: AsRef<Path>>(source_file: SourceFile, destination_dir_path: D, multi: &MultiProgress) -> Self {
    let destination_dir = destination_dir_path.as_ref().to_path_buf();

    let progress_bar = MyProgressBar::new(multi);

    Self {
      source_file,
      destination_dir_path: destination_dir,
      progress_bar,
    }
  }

  pub fn source_file_name(&self) -> String {
    self.source_file.file_name()
  }

  pub fn destination_file(&self) -> PathBuf {
    self.destination_dir_path.join(self.source_file.relative_path())
  }

  pub async fn copy<'a>(self, buffer: BufferSize, mux: MonitorMux) -> R<()> {
    let progress_bar = &self.progress_bar;
    progress_bar.tick();
    progress_bar.set_prefix(self.source_file_name());
    let file_name = &self.source_file.file_name();

    mux.send_not_started(progress_bar).await;

    let mut source_file = Self::open_source_file(&self.source_file.full_path(), &mux, progress_bar).await?;
    let file_size = Self::get_file_length(file_name, &source_file, FileType::Source, &mux, progress_bar).await?;
    Self::create_destination_path(&self.destination_file(), &mux, progress_bar).await?;
    let mut destination_file = Self::create_destination_file(&self.destination_file(), &mux, progress_bar).await?;

    progress_bar.set_file_size(file_size);
    let buf_size =
      if file_size <= buffer.bytes() {
        file_size as usize // If file_size can be contained in buffer, then use that as the buffer size and don't chunk
      } else {
        buffer.bytes() as usize // If the file_size can't be contained in buffer, then chunk by buffer size
      };

    let mut buffer = vec![0; buf_size];

    loop {
      let bytes_read = Self::read_to_buffer(file_name, &mut source_file, &mut buffer, &mux, progress_bar).await?;

      if bytes_read == 0 {
        Self::complete_file_copy(file_name, &mut destination_file, file_size, &mux, progress_bar, self.source_file_name().as_str()).await?;
        return Ok(())
      }

      Self::write_to_destination(file_name, &mut destination_file, &buffer[..bytes_read as usize], &mux, progress_bar).await?;
    }
  }

  async fn open_source_file<P: AsRef<Path> + Clone>(file: P, mux: &MonitorMux, progress_bar: &MyProgressBar) -> R<File> {
      match File::open(file.as_ref()).await {
        Ok(file) => {
          mux.send_opened_source_file(progress_bar).await;
          Ok(file)
        },
        Err(e) => {
          mux.send_could_not_read_source_file(file, <std::io::Error as Into<CopyError>>::into(e), progress_bar).await;
          Err(())
        }
      }
  }

  async fn get_file_length(file_name: &str, file: &File, file_type: FileType, mux: &MonitorMux, progress_bar: &MyProgressBar) -> R<u64> {

    mux.send_getting_file_length(&file_type, progress_bar).await;

      match file.metadata().await {
        Ok(meta) => {
          mux.send_got_file_length(&file_type, progress_bar).await;
          Ok(meta.len())
        },
        Err(e) => {
          mux.send_could_not_get_file_size(file_name, &file_type, <std::io::Error as Into<CopyError>>::into(e), progress_bar).await;
          Err(())
        }
      }
  }


  async fn create_destination_path<P: AsRef<Path> + Clone>(destination_file: P, mux: &MonitorMux, progress_bar: &MyProgressBar) -> R<()> {
    if let Some(parent_path) = destination_file.as_ref().parent() {
    // check if it exists, if not create it
     if !parent_path.exists() {
       let result =
         DirBuilder::new()
          .recursive(true)
          .create(parent_path)
          .await;

        if let Err(e) = result {
          mux.send_could_not_create_destination_directory(destination_file, <std::io::Error as Into<CopyError>>::into(e), progress_bar).await;
          return Err(());
        }
     }
    }

    Ok(())
  }

  async fn create_destination_file<P: AsRef<Path> + Clone>(destination_file: P, mux: &MonitorMux, progress_bar: &MyProgressBar) -> R<File> {
    match File::create(destination_file.as_ref()).await {
      Ok(df) => {
        mux.send_created_destination_file(progress_bar).await;
        Ok(df)
      },
      Err(e) => {
        mux.send_could_not_create_destination_file(destination_file, <std::io::Error as Into<CopyError>>::into(e), progress_bar).await;
        Err(())
      }
    }
  }

  async fn read_to_buffer(file: &str, source_file: &mut File, buffer: &mut [u8], mux: &MonitorMux, progress_bar: &MyProgressBar) -> R<u64> {
    let bytes_read_result =
      source_file
        .read(buffer)
        .await;

    match bytes_read_result {
      Ok(value) => Ok(value as u64),
      Err(e) => {
        mux.send_read_failed(file, <std::io::Error as Into<CopyError>>::into(e), progress_bar).await;
        Err(())
      }
    }
  }

  async fn write_to_destination(file: &str, destination_file: &mut File, read_buffer: &[u8], mux: &MonitorMux, progress_bar: &MyProgressBar) -> R<()> {
    let bytes_written_result =
      destination_file
        .write(read_buffer)
        .await;

    let bytes_written = match bytes_written_result {
      Ok(value) => value as u64,
      Err(e) => {
        mux.send_write_to_destination_failed(file, <std::io::Error as Into<CopyError>>::into(e), progress_bar).await;
        return Err(())
      }
    };

    mux.send_copy_in_progress(bytes_written, progress_bar).await;

    Ok(())
  }

  async fn complete_file_copy(file: &str, destination_file: &mut File, file_size: u64, mux: &MonitorMux, progress_bar: &MyProgressBar, file_name: &str) -> R<()> {

    mux.send_flushing_destination_file(progress_bar).await;
    let flush_result = destination_file.flush().await;

    match flush_result {
      Ok(_) => (),
      Err(e) => mux.send_flushing_to_destination_file_failed(file, <std::io::Error as Into<CopyError>>::into(e), progress_bar).await,
    }

    mux.send_copy_complete(progress_bar).await;

    let dest_file_size = Self::get_file_length(file, destination_file, FileType::Destination, mux, progress_bar).await?;

    Self::compare_file_sizes(file, file_size, dest_file_size, mux, progress_bar).await?;
    Self::succeed(mux, progress_bar, file_name, file_size).await?;

    Ok(())
  }


  async fn compare_file_sizes(file: &str, source_file_size: u64, destination_file_size: u64, mux: &MonitorMux, progress_bar: &MyProgressBar) -> R<()> {
    if source_file_size == destination_file_size {
      mux.send_file_sizes_match(progress_bar).await
    } else {
      let size_comparison = SizeComparison::new(source_file_size, destination_file_size);
      mux.send_files_sizes_are_different(file, size_comparison, progress_bar).await
    }

    Ok(())
  }

  async fn succeed(mux: &MonitorMux, progress_bar: &MyProgressBar, file_name: &str, file_size: u64) -> R<()> {
    mux.send_success(file_name, file_size, progress_bar).await;
    Ok(())
  }
}
