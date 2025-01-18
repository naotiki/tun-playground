
use std::ffi::CStr;
use std::{io, net::Ipv4Addr};

use std::pin::Pin;
use std::task::{ready, Context, Poll};
use tappers::{Interface, Tap};
use tokio::io::unix::AsyncFd;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

pub struct AsyncTap {
    tap: AsyncFd<Tap>,
}

impl AsyncTap {
    pub fn new() -> io::Result<Self> {
        let mut tap = Tap::new()?;
        let iface = Interface::from_cstr(unsafe { CStr::from_bytes_with_nul_unchecked(b"tap0\0") }).unwrap();
        tap.add_addr(Ipv4Addr::new(10, 0, 0, 1))?;
        tap.set_up()?;
        /* let tap_writer = Arc::new(tap);
        let tap_reader = tap_writer.clone(); */
        let async_fd = AsyncFd::new(tap)?;
        Ok(Self { tap: async_fd })
    }
}

impl AsyncRead for AsyncTap {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        loop {
            let mut guard = ready!(self.tap.poll_read_ready(cx))?;

            let unfilled = buf.initialize_unfilled();
            match guard.try_io(|inner| inner.get_ref().recv(unfilled)) {
                Ok(Ok(len)) => {
                    buf.advance(len);
                    return Poll::Ready(Ok(()));
                }
                Ok(Err(err)) => return Poll::Ready(Err(err)),
                Err(_would_block) => continue,
            }
        }
    }
}

impl AsyncWrite for AsyncTap {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        loop {
            let mut guard = ready!(self.tap.poll_write_ready(cx))?;

            match guard.try_io(|inner| inner.get_ref().send(buf)) {
                Ok(result) => return Poll::Ready(result),
                Err(_would_block) => continue,
            }
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }
}
