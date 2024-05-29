use std::path::{Path, PathBuf};
use indicatif::MultiProgress;
use tokio::fs::{DirBuilder, File};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::mpsc::Sender;

use crate::progress::MyProgressBar;
use crate::model::{Complete, FailedReason, FileStatus, FileType, InProgress, R};

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
      progress_bar: progress_bar,
    }
  }

  pub fn source_file_name(&self) -> String {
    self.source_file.file_name()
  }

  pub fn destination_file(&self) -> PathBuf {
    self.destination_dir_path.join(self.source_file.relative_path())
  }

  pub async fn copy<'a>(self, tx: Sender<FileStatus>) -> R<()> {
    let progress_bar = &self.progress_bar;
    progress_bar.tick();
    progress_bar.set_prefix(self.source_file_name());

    let _ =
      tx
        .send(FileStatus::NotStarted(progress_bar.clone()))
        .await;

    let mut source_file = Self::open_source_file(&self.source_file.full_path(), &tx, progress_bar).await?;
    let file_size = Self::get_file_length(&source_file, FileType::Source, &tx, progress_bar).await?;
    let _ = Self::create_destination_path(&self.destination_file(), &tx, progress_bar).await?;
    let mut destination_file = Self::create_destination_file(&self.destination_file(), &tx, progress_bar).await?;

    progress_bar.set_file_size(file_size);
    let progress_bar = progress_bar.clone();
    let buf_size = 1024 * 1024;
    let mut buffer = vec![0; buf_size];

    loop {
      let bytes_read = Self::read_to_buffer(&mut source_file, &mut buffer, &tx, &progress_bar).await?;

      if bytes_read == 0 {
        Self::complete_file_copy(&mut destination_file, file_size, &tx, &progress_bar).await?;
        return Ok(())
      }

      Self::write_to_destination(&mut destination_file, &buffer[..bytes_read as usize], &tx, &progress_bar).await?;
    }
  }

  async fn open_source_file<P: AsRef<Path>>(file: P, tx: &Sender<FileStatus>, progress_bar: &MyProgressBar) -> R<File> {
      match File::open(file.as_ref()).await {
        Ok(file) => {
          let _ = tx.send(FileStatus::OpenedSourceFile(progress_bar.clone())).await;
          Ok(file)
        },
        Err(e) => {
          let _ = tx.send(FileStatus::Failed(FailedReason::CouldNotReadSourceFile(e.to_string(), progress_bar.clone()))).await;
          Err(())
        }
      }
  }

  async fn get_file_length(file: &File, file_type: FileType, tx: &Sender<FileStatus>, progress_bar: &MyProgressBar) -> R<u64> {

      let _ = tx.send(FileStatus::GettingFileLength(file_type.clone(), progress_bar.clone())).await;

      match file.metadata().await {
        Ok(meta) => {
          let _ = tx.send(FileStatus::GotFileLength(file_type, progress_bar.clone())).await;
          Ok(meta.len())
        },
        Err(e) => {
          let _ = tx.send(FileStatus::Failed(FailedReason::CouldNotGetFileSize(e.to_string(), file_type, progress_bar.clone()))).await;
          Err(())
        }
      }
  }


  async fn create_destination_path<P: AsRef<Path>>(destination_file: P, tx: &Sender<FileStatus>, progress_bar: &MyProgressBar) -> R<()> {
       if let Some(parent_path) = destination_file.as_ref().parent() {
        // check if it exists, if not create it
         if !parent_path.exists() {
           let result =
             DirBuilder::new()
              .recursive(true)
              .create(parent_path)
              .await;

            if let Err(e) = result {
              let _ = tx.send(FileStatus::Failed(FailedReason::CouldNotCreateDestinationDir(e.to_string(), progress_bar.clone()))).await;
              return Err(());
            }
         }
       }

       Ok(())
  }

  async fn create_destination_file<P: AsRef<Path>>(destination_file: P, tx: &Sender<FileStatus>, progress_bar: &MyProgressBar) -> R<File> {
        match File::create(destination_file.as_ref()).await {
          Ok(df) => {
            let _ = tx.send(FileStatus::CreatedDestinationFile(progress_bar.clone())).await;
            Ok(df)
          },
          Err(e) => {
            let _ = tx.send(FileStatus::Failed(FailedReason::CouldNotCreateDestinationFile(e.to_string(), progress_bar.clone()))).await;
            return Err(());
          }
        }
  }

  async fn read_to_buffer(source_file: &mut File, buffer: &mut [u8], tx: &Sender<FileStatus>, progress_bar: &MyProgressBar) -> R<u64> {
    let bytes_read_result =
      source_file
        .read(buffer)
        .await;

    match bytes_read_result {
      Ok(value) => Ok(value as u64),
      Err(e) => {
        let _ = tx.send(FileStatus::Failed(FailedReason::ReadFailed(e.to_string(), progress_bar.clone()))).await;
        return Err(());
      }
    }
  }

  async fn write_to_destination(destination_file: &mut File, read_buffer: &[u8], tx: &Sender<FileStatus>, progress_bar: &MyProgressBar) -> R<()> {
      let bytes_written_result =
        destination_file
          .write(&read_buffer)
          .await;

      let bytes_written = match bytes_written_result {
        Ok(value) => value as u64,
        Err(e) => {
          let _ = tx.send(FileStatus::Failed(FailedReason::WriteFailed(e.to_string(), progress_bar.clone()))).await;

          return Err(())
        }
      };

      let _ = tx.send(
        FileStatus::CopyInProgress(
          InProgress::new(bytes_written, progress_bar)
        )
      ).await;

      Ok(())
  }

  async fn complete_file_copy(destination_file: &mut File, file_size: u64, tx: &Sender<FileStatus>, progress_bar: &MyProgressBar) -> R<()> {

        let _ = tx.send(FileStatus::Flushing(progress_bar.clone())).await;

        let _ = destination_file
          .flush()
          .await
          .or_else(|e| {
            let _ = tx.send(FileStatus::Failed(FailedReason::FlushFailed(e.to_string(), progress_bar.clone())));
            return Err(())
          });

        let _ = tx.send(FileStatus::CopyComplete(Complete::new(progress_bar))).await;

        let dest_file_size = Self::get_file_length(&destination_file, FileType::Destination, &tx, &progress_bar).await?;

        Self::compare_file_sizes(file_size, dest_file_size, &tx, &progress_bar).await?;

        Ok(())
  }


    async fn compare_file_sizes(source_file_size: u64, destination_file_size: u64, tx: &Sender<FileStatus>, progress_bar: &MyProgressBar) -> R<()> {

    if source_file_size == destination_file_size {
      let _ = tx.send(FileStatus::FileSizesMatch(progress_bar.clone())).await;
    } else {
      let _ = tx.send(FileStatus::Failed(FailedReason::FileSizesAreDifferent(source_file_size, destination_file_size, progress_bar.clone()))).await;
    }

    Ok(())
  }
}
