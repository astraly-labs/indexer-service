/// TAKEN FROM : https://github.com/hominee/streamer
use std::borrow::Cow;
use std::fs::File;
use std::io;
use std::io::Read;
use std::pin::Pin;
use std::task::{Context, Poll};

use futures_util::stream::{Stream, StreamExt};
use hyper::body::Bytes;
use hyper::Body;

pub struct Streaming<T> {
    inner: T,
    offset: usize,
    len: usize,
}

impl<T> Streaming<T> {
    pub fn new(data: T) -> Self {
        Self { inner: data, offset: 0, len: 0 }
    }
}

impl From<File> for Streaming<File> {
    fn from(file: File) -> Self {
        Self { len: file.metadata().unwrap().len() as _, inner: file, offset: 0 }
    }
}

impl Stream for Streaming<File> {
    type Item = u8;
    fn poll_next(mut self: Pin<&mut Self>, _ctx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut byte = 0;
        let mut r = Poll::Ready(None);
        if let Ok(size) = self.as_mut().inner.read(std::slice::from_mut(&mut byte)) {
            if size > 0 {
                // println!("offset {:?}, buf: {:?}", offset, buf);
                r = Poll::Ready(Some(byte))
            }
        }
        r
        // stream::poll_fn(move |_| -> Poll<Option<&'static [u8]>> {
        //    match f.read(&mut buf) {
        //        Ok(size) => {
        //            if size > 0 {
        //                println!("{:?}", &size);
        //                Poll::Ready(Some(buf))
        //            } else {
        //                println!("{:?}", "EOF");
        //                Poll::Ready(None)
        //            }
        //        }
        //        Err(_) => Poll::Ready(None),
        //    }
        // })
        // .flat_map(|e| stream::iter(e))
        // .chunks(5)
    }
}

#[test]
fn test_stream_file() {
    use futures_util::StreamExt;
    async fn run() {
        let file = File::open("markdown-tools.js").unwrap();
        let streaming = Streaming { len: file.metadata().unwrap().len() as _, offset: 0, inner: file };
        streaming
            .chunks(5)
            .take(3)
            .for_each(|en| async move {
                println!("{:?}", &en);
            })
            .await;
    }

    use tokio::runtime::Builder;
    let _rt = Builder::new_current_thread().enable_all().build().unwrap();
    //_rt.block_on(run());
    // assert!(false);
}

impl<T: Clone, const N: usize> From<[T; N]> for Streaming<Cow<'static, [T]>> {
    fn from(s: [T; N]) -> Self {
        Self { len: s.len(), inner: Cow::Owned(Box::<[T]>::from(s).into_vec()), offset: 0 }
    }
}

impl<T: Clone> From<Vec<T>> for Streaming<Cow<'static, Vec<T>>> {
    fn from(s: Vec<T>) -> Self {
        Self { len: s.len(), inner: Cow::Owned(s), offset: 0 }
    }
}

impl<T: Clone + Copy + Unpin> Stream for Streaming<Cow<'static, Vec<T>>> {
    type Item = T;
    fn poll_next(mut self: Pin<&mut Self>, _ctx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut r = Poll::Ready(None);
        let offset = self.offset;
        if offset > self.len - 1 {
            return r;
        }
        if let Cow::Owned(ref own) = self.as_mut().inner {
            r = Poll::Ready(Some(own[offset]));
            self.get_mut().offset += 1;
        }
        r
    }
}

impl From<&'static str> for Streaming<Cow<'static, [u8]>> {
    fn from(s: &'static str) -> Self {
        Self { len: s.len(), inner: Cow::Borrowed(s.as_bytes()), offset: 0 }
    }
}

impl From<String> for Streaming<Cow<'static, [u8]>> {
    fn from(s: String) -> Self {
        Self { len: s.len(), inner: Cow::Owned(s.into_bytes()), offset: 0 }
    }
}

impl Stream for Streaming<Cow<'static, [u8]>> {
    type Item = u8;
    fn poll_next(mut self: Pin<&mut Self>, _ctx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut r = Poll::Ready(None);
        let offset = self.offset;
        if offset > self.len - 1 {
            return r;
        }
        match self.as_mut().inner {
            Cow::Owned(ref mut own) => {
                r = Poll::Ready(Some(own[offset]));
                self.as_mut().offset += 1;
            }
            Cow::Borrowed(bow) => {
                r = Poll::Ready(Some(bow[offset]));
                self.as_mut().offset += 1;
            }
        }

        r
    }
}

#[test]
fn test_stream_byte() {
    use futures_util::StreamExt;
    async fn run() {
        let file: [u64; 11] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 00];
        let streaming = Streaming::from(Box::from(file));
        streaming
            .chunks(4)
            .take(3)
            .for_each(|en| async move {
                println!("{:?}", &en);
            })
            .await;
        let file: [u8; 8] = [1, 2, 3, 4, 5, 6, 7, 8];
        let streaming = Streaming::from(file);
        streaming
            .chunks(4)
            .take(3)
            .for_each(|en| async move {
                println!("{:?}", &en);
            })
            .await;
        let file = "a very long string though";
        let streaming = Streaming::from(file);
        streaming
            .chunks(10)
            .take(3)
            .for_each(|en| async move {
                println!("{:?}", std::str::from_utf8(&en));
            })
            .await;
        let file = String::from("a very long string though");
        let streaming = Streaming::from(file);
        streaming
            .chunks(8)
            .take(3)
            .for_each(|en| async move {
                println!("{:?}", std::str::from_utf8(&en));
            })
            .await;
        let file = vec![11, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let streaming = Streaming::from(file);
        streaming
            .chunks(4)
            .take(3)
            .for_each(|en| async move {
                println!("{:?}", &en);
            })
            .await;
    }

    use tokio::runtime::Builder;
    let _rt = Builder::new_current_thread().enable_all().build().unwrap();
    //_rt.block_on(run());
    // assert!(false);
}

impl<T> From<Box<[T]>> for Streaming<Cow<'static, Vec<T>>>
where
    T: Clone,
{
    fn from(s: Box<[T]>) -> Self {
        Self { len: s.len(), inner: Cow::Owned(s.into_vec()), offset: 0 }
    }
}

pub struct Streamer<T> {
    src: Streaming<T>,
    pub meta: Meta,
}

pub struct Meta {
    name: Option<String>,
    filename: Option<String>,
    boundary: Boundary,
    buf_len: usize,
}
impl Meta {
    pub fn set_filename<T: Into<String>>(&mut self, filename: T) {
        self.filename = Some(filename.into());
    }

    pub fn set_name<T: Into<String>>(&mut self, name: T) {
        self.name = Some(name.into());
    }

    pub fn set_buf_len(&mut self, buf_len: usize) {
        self.buf_len = buf_len;
    }

    fn ser_name(&self) -> String {
        if self.name.is_none() { "".into() } else { format!(" filename=\"{}\";", self.name.as_ref().unwrap()) }
    }

    fn ser_filename(&self, ind: usize) -> String {
        if self.filename.is_none() {
            "".into()
        } else {
            format!(" name=\"{}.{}\";", self.filename.as_ref().unwrap(), ind)
        }
    }

    pub fn write_head(&self, ind: usize) -> Bytes {
        let s = format!(
            "--{}\r\nContent-Disposition: form-data;{}{}\r\n\r\n",
            self.boundary.to_str(),
            self.ser_name(),
            self.ser_filename(ind)
        );
        Bytes::from(s)
    }

    pub fn write_tail(&self, body_len: usize) -> Bytes {
        let mut s = "\r\n".into();
        if body_len < self.buf_len {
            s = format!("\r\n--{}--\r\n", self.boundary.to_str());
        }
        Bytes::from(s)
    }
}

impl<T> Streamer<T>
where
    // T: Stream + Send + 'static,
    Streaming<T>: Stream + Send + 'static,
    <Streaming<T> as Stream>::Item: Send + Into<u8>,
    Bytes: From<Vec<<Streaming<T> as Stream>::Item>>,
{
    pub fn new<P>(src: P) -> Self
    where
        Streaming<T>: From<P>,
    {
        let boundary = gen_boundary("1234567890abcdefghijklmnopqrstuvw");
        let src = Streaming::<T>::from(src);
        Self { src, meta: Meta { name: None, filename: None, boundary, buf_len: 64 * 1024 } }
    }

    pub fn streaming(self) -> Body {
        let (src, meta) = (self.src, self.meta);
        let mut ind = 0;
        let stream = src
            .chunks(meta.buf_len)
            .map(move |ck| {
                let head = meta.write_head(ind);
                ind += 1;
                let tail = meta.write_tail(ck.len());
                let stream_head = futures_util::stream::once(async { Ok::<_, io::Error>(head) });
                let stream_body = futures_util::stream::once(async { Ok::<_, io::Error>(Bytes::from(ck)) });
                let stream_tail = futures_util::stream::once(async { Ok::<_, io::Error>(tail) });
                stream_head.chain(stream_body).chain(stream_tail)
            })
            .flatten();

        Body::wrap_stream(stream)
    }
}

#[test]
fn test_streamer() {
    use futures_util::StreamExt;

    async fn run() {
        // let file: [u8; 11] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 00];
        let file = std::fs::File::open("info").unwrap();
        let mut streaming = Streamer::new(file);
        streaming.meta.set_buf_len(10);
        streaming.meta.set_name("doc");
        streaming.meta.set_filename("info");
        streaming
            .streaming()
            .take(100)
            .for_each(|en| async move {
                println!("{:?}", &en);
            })
            .await;
    }

    use tokio::runtime::Builder;
    let _rt = Builder::new_current_thread().enable_all().build().unwrap();
    //_rt.block_on(run());
    // assert!(false);
}

pub struct Boundary(pub [u8; 32]);
impl Boundary {
    pub fn to_str(&self) -> String {
        String::from_utf8_lossy(&self.0).into()
    }
}
impl From<&Meta> for Boundary {
    fn from(meta: &Meta) -> Self {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::Hasher;

        let mut inner = [0; 32];
        let mut hasher = DefaultHasher::new();
        inner.iter_mut().for_each(|e| {
            hasher.write_usize(meta.buf_len);
            hasher.write(meta.filename.as_ref().unwrap_or(&"".into()).as_bytes());
            hasher.write(meta.name.as_ref().unwrap_or(&"".into()).as_bytes());
            let raw_ind = hasher.finish() % 36;
            if raw_ind < 10 {
                *e = raw_ind as u8 + 48;
            } else {
                *e = raw_ind as u8 - 10 + 97;
            }
        });
        Boundary(inner)
    }
}
pub fn gen_boundary<T: AsRef<[u8]>>(del: T) -> Boundary {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::Hasher;

    let mut inner = [0; 32];
    let mut hasher = DefaultHasher::new();
    inner.iter_mut().for_each(|e| {
        hasher.write(del.as_ref());
        let raw_ind = hasher.finish() % 36;
        if raw_ind < 10 {
            *e = raw_ind as u8 + 48;
        } else {
            *e = raw_ind as u8 - 10 + 97;
        }
    });
    Boundary(inner)
}

//    fn ser_data(&mut self, sender: &mut Sender) {
//        use std::io::Read;
//        let del = self.len - self.offset;
//        if del < self.buf_len && del > 0 {
//            let mut data = Vec::new();
//            self.file.read_to_end(&mut data).unwrap();
//            self.offset += data.len();
//            sender.send_data(Bytes::from(data));
//            let tail = format!("\r\n--{}", self.boundary.to_str());
//            sender.send_data(Bytes::from(tail));
//            return;
//        }
//
//        let mut data = Vec::with_capacity(self.buf_len);
//        unsafe {
//            data.set_len(self.buf_len);
//        };
//        self.file.read(&mut data).unwrap();
//        sender.send_data(Bytes::from(data));
//        let tail = format!("\r\n--{}", self.boundary.to_str());
//        sender.send_data(Bytes::from(tail));
//        self.offset += self.buf_len;
//    }
//
//    async fn write_body(&mut self) -> Body {
//        let (mut sender, body) = Body::channel();
//        let head = format!("--{}\r\n", self.boundary.to_str());
//        sender.send_data(Bytes::from(head));
//
//        let dcp = format!(
//            "Content-Disposition: form-data;{}{}\r\n",
//            self.ser_name(),
//            self.ser_filename()
//        );
//        sender.send_data(Bytes::from(dcp));
//        sender.send_data(Bytes::from("\r\n"));
//
//        // write the body
//        self.ser_data(&mut sender);
//
//        body
//
//        //use std::io::Read;
//        //sender.send_data(Bytes::from(data));
//        //write!(data, "\r\n")?; // The key thing you are missing
//        //sender.send_data("\r\n--{}--\r\n", self.boundary)?;
//    }
