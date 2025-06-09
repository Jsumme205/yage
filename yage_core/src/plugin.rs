use core::{task::{Poll, Context}, pin::Pin};

pub trait Plugin<In> {
  type Output;
  type Error;

  fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>, input: In) -> Poll<Result<(), Self::Error>>;

  fn poll_transform(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<Self::Output, Self::Error>>;
}