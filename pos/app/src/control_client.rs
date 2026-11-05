use anyhow::{bail, Context, Result};
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

pub struct ControlClient {
    reader: BufReader<TcpStream>,
    writer: BufWriter<TcpStream>,
    next_id: u64,
}

impl ControlClient {
    pub fn connect(addr: impl ToSocketAddrs) -> Result<ControlClient> {
        Self::connect_timeout(addr, Duration::from_secs(2))
    }

    pub fn connect_timeout(addr: impl ToSocketAddrs, timeout: Duration) -> Result<ControlClient> {
        let addr = addr
            .to_socket_addrs()?
            .next()
            .context("control address resolved to nothing")?;
        let stream = TcpStream::connect_timeout(&addr, timeout).context("TCP connect failed")?;
        stream.set_read_timeout(Some(Duration::from_secs(10)))?;
        stream.set_write_timeout(Some(Duration::from_secs(5)))?;
        stream.set_nodelay(true)?;
        let reader = BufReader::new(stream.try_clone().context("clone socket")?);
        let writer = BufWriter::new(stream);
        Ok(ControlClient { reader, writer, next_id: 1 })
    }

    pub fn call(&mut self, method: &str, params: Value) -> Result<Value> {
        let id = self.next_id;
        self.next_id += 1;
        let line = json!({"id": id, "method": method, "params": params});
        writeln!(self.writer, "{line}").context("write request")?;
        self.writer.flush().context("flush request")?;

        loop {
            let mut buf = String::new();
            let n = self.reader.read_line(&mut buf).context("read reply")?;
            if n == 0 {
                bail!("connection closed by designcraft (bad request closes it; reconnect)");
            }
            let v: Value = serde_json::from_str(buf.trim())
                .with_context(|| format!("bad reply line: {buf:.200}"))?;
            let reply_id = v.get("id").and_then(Value::as_u64);
            match reply_id {
                Some(rid) if rid != id => continue,
                _ => return self.finish_reply(v),
            }
        }
    }

    fn finish_reply(&self, v: Value) -> Result<Value> {
        if v.get("ok").and_then(Value::as_bool).unwrap_or(false) {
            Ok(v.get("result").cloned().unwrap_or(Value::Null))
        } else {
            let err = v
                .get("error")
                .and_then(Value::as_str)
                .unwrap_or("unknown error");
            bail!("designcraft error: {err}");
        }
    }

    pub fn inspect(&mut self) -> Result<Value> {
        self.call("document.inspect", json!({}))
    }

    pub fn execute(&mut self, command: &str, params: &Value) -> Result<Value> {
        self.call(command, params.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_and_error_reply() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let server = std::thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut writer = BufWriter::new(stream);
            for id in 1..=2u64 {
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                let req: Value = serde_json::from_str(line.trim()).unwrap();
                let method = req["method"].as_str().unwrap().to_owned();
                let reply = if id == 1 {
                    json!({"id": id, "ok": true, "result": {"method": method}})
                } else {
                    json!({"id": id, "ok": false, "error": "no document"})
                };
                writeln!(writer, "{reply}").unwrap();
                writer.flush().unwrap();
            }
        });

        let mut c = ControlClient::connect(("127.0.0.1", port)).unwrap();
        let r = c.call("engine.commands", json!({})).unwrap();
        assert_eq!(r["method"], "engine.commands");
        let err = c.call("document.inspect", json!({})).unwrap_err();
        assert!(err.to_string().contains("no document"));
        server.join().unwrap();
    }
}
