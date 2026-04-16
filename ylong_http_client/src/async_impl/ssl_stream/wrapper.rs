// Copyright (c) 2023 Huawei Device Co., Ltd.
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use core::pin::Pin;
use core::task::{Context, Poll};
use std::io::{self, Read, Write};

use crate::c_openssl::bio::Wrapper;
use crate::runtime::{AsyncRead, AsyncWrite, ReadBuf};

impl<S> Wrapper<S> {
    pub(crate) fn with_context(mut self, ctx: &mut Context<'_>) -> Self {
        self.context = ctx as *mut _ as *mut ();
        self
    }

    pub(crate) fn clear_context(&mut self) {
        self.context = ptr::null_mut();
    }

    unsafe fn inner(&mut self) -> (Pin<&mut S>, &mut Context<'_>) {
        debug_assert!(!self.context.is_null());
        let stream = Pin::new_unchecked(&mut self.stream);
        let context = &mut *(self.context as *mut _);
        (stream, context)
    }
}

impl<S> Read for Wrapper<S>
where
    S: AsyncRead,
{
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let (stream, cx) = unsafe { self.inner() };
        let mut buf = ReadBuf::new(buf);
        match stream.poll_read(cx, &mut buf)? {
            Poll::Ready(()) => Ok(buf.filled().len()),
            Poll::Pending => Err(io::Error::from(io::ErrorKind::WouldBlock)),
        }
    }
}

impl<S> Write for Wrapper<S>
where
    S: AsyncWrite,
{
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let (stream, cx) = unsafe { self.inner() };
        match stream.poll_write(cx, buf) {
            Poll::Ready(r) => r,
            Poll::Pending => Err(io::Error::from(io::ErrorKind::WouldBlock)),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        let (stream, cx) = unsafe { self.inner() };
        match stream.poll_flush(cx) {
            Poll::Ready(r) => r,
            Poll::Pending => Err(io::Error::from(io::ErrorKind::WouldBlock)),
        }
    }
}

use std::ptr;

pub(crate) fn check_io_to_poll<T>(r: io::Result<T>) -> Poll<io::Result<T>> {
    match r {
        Ok(t) => Poll::Ready(Ok(t)),
        Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => Poll::Pending,
        Err(e) => Poll::Ready(Err(e)),
    }
}
