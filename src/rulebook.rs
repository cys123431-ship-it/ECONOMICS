use flate2::read::GzDecoder;
use std::io::{self, BufRead, BufReader, Cursor, Read};

pub const EXPECTED_RULES: usize = 27_494;
pub const RAW_SHA256: &str = "2f2a3a189c594fdb2a581e6f052123a0dc778e8065677e88d5764f9c813b0b56";
static RULEBOOK_GZ: &[u8] = include_bytes!("../assets/Market_Economy_Radar_Rulebook_v4_ULTRA.txt.gz");

#[derive(Debug, Clone)]
pub struct RuleHit { pub id:String, pub priority:String, pub scope:String, pub severity:String, pub title:String, pub message:String }

pub fn reader() -> BufReader<GzDecoder<Cursor<&'static [u8]>>> {
    BufReader::with_capacity(64*1024, GzDecoder::new(Cursor::new(RULEBOOK_GZ)))
}

pub fn count_rules() -> io::Result<usize> {
    let mut n=0usize; let mut line=String::new(); let mut r=reader();
    while r.read_line(&mut line)? != 0 { if line.starts_with("RULE\t") { n+=1; } line.clear(); }
    Ok(n)
}

pub fn raw_sha256_via_system_free() -> io::Result<String> {
    let mut sha=Sha256::new(); let mut r=reader(); let mut buf=[0u8;65536];
    loop { let n=r.read(&mut buf)?; if n==0{break;} sha.update(&buf[..n]); }
    Ok(sha.finish_hex())
}

pub fn verify() -> io::Result<()> {
    let rules=count_rules()?;
    if rules!=EXPECTED_RULES { return Err(io::Error::new(io::ErrorKind::InvalidData, format!("rule count mismatch: {rules}"))); }
    let sha=raw_sha256_via_system_free()?;
    if sha!=RAW_SHA256 { return Err(io::Error::new(io::ErrorKind::InvalidData, format!("rulebook SHA mismatch: {sha}"))); }
    println!("rules={rules} sha256={sha} gzip_bytes={}", RULEBOOK_GZ.len());
    Ok(())
}

pub fn stream_evaluate<F>(mut predicate:F, max_hits:usize) -> io::Result<(usize,Vec<RuleHit>)>
where F: FnMut(&str)->bool {
    let mut r=reader(); let mut line=String::new(); let mut pending:Option<(String,String,String,String,String)>=None;
    let mut evaluated=0usize; let mut hits=Vec::new();
    while r.read_line(&mut line)?!=0 {
        if line.starts_with("RULE\t") {
            evaluated+=1;
            let cols:Vec<&str>=line.trim_end().split('\t').collect();
            if cols.len()>=7 && predicate(cols[5]) {
                pending=Some((cols[1].to_string(),cols[2].to_string(),cols[3].to_string(),cols[4].to_string(),cols[5].to_string()));
            } else { pending=None; }
        } else if line.starts_with("MSG\t") {
            if let Some((id,priority,scope,severity,_cond))=pending.take() {
                if hits.len()<max_hits {
                    let cols:Vec<&str>=line.trim_end().split('\t').collect();
                    hits.push(RuleHit{id,priority,scope,severity,title:cols.get(1).copied().unwrap_or("").to_string(),message:cols.get(2).copied().unwrap_or("").to_string()});
                }
            }
        }
        line.clear();
    }
    Ok((evaluated,hits))
}

struct Sha256 { h:[u32;8], buf:[u8;64], len:u64, used:usize }
impl Sha256 {
    fn new()->Self{Self{h:[0x6a09e667,0xbb67ae85,0x3c6ef372,0xa54ff53a,0x510e527f,0x9b05688c,0x1f83d9ab,0x5be0cd19],buf:[0;64],len:0,used:0}}
    fn update(&mut self,data:&[u8]){for &b in data{self.buf[self.used]=b;self.used+=1;self.len+=8;if self.used==64{self.compress();self.used=0;}}}
    fn finish_hex(mut self)->String{let bit_len=self.len;self.buf[self.used]=0x80;self.used+=1;if self.used>56{while self.used<64{self.buf[self.used]=0;self.used+=1;}self.compress();self.used=0;}while self.used<56{self.buf[self.used]=0;self.used+=1;}self.buf[56..64].copy_from_slice(&bit_len.to_be_bytes());self.compress();self.h.iter().map(|x|format!("{x:08x}")).collect()}
    fn compress(&mut self){const K:[u32;64]=[0x428a2f98,0x71374491,0xb5c0fbcf,0xe9b5dba5,0x3956c25b,0x59f111f1,0x923f82a4,0xab1c5ed5,0xd807aa98,0x12835b01,0x243185be,0x550c7dc3,0x72be5d74,0x80deb1fe,0x9bdc06a7,0xc19bf174,0xe49b69c1,0xefbe4786,0x0fc19dc6,0x240ca1cc,0x2de92c6f,0x4a7484aa,0x5cb0a9dc,0x76f988da,0x983e5152,0xa831c66d,0xb00327c8,0xbf597fc7,0xc6e00bf3,0xd5a79147,0x06ca6351,0x14292967,0x27b70a85,0x2e1b2138,0x4d2c6dfc,0x53380d13,0x650a7354,0x766a0abb,0x81c2c92e,0x92722c85,0xa2bfe8a1,0xa81a664b,0xc24b8b70,0xc76c51a3,0xd192e819,0xd6990624,0xf40e3585,0x106aa070,0x19a4c116,0x1e376c08,0x2748774c,0x34b0bcb5,0x391c0cb3,0x4ed8aa4a,0x5b9cca4f,0x682e6ff3,0x748f82ee,0x78a5636f,0x84c87814,0x8cc70208,0x90befffa,0xa4506ceb,0xbef9a3f7,0xc67178f2];let mut w=[0u32;64];for (i,c) in self.buf.chunks_exact(4).take(16).enumerate(){w[i]=u32::from_be_bytes([c[0],c[1],c[2],c[3]]);}for i in 16..64{let s0=w[i-15].rotate_right(7)^w[i-15].rotate_right(18)^(w[i-15]>>3);let s1=w[i-2].rotate_right(17)^w[i-2].rotate_right(19)^(w[i-2]>>10);w[i]=w[i-16].wrapping_add(s0).wrapping_add(w[i-7]).wrapping_add(s1);}let(mut a,mut b,mut c,mut d,mut e,mut f,mut g,mut h)=(self.h[0],self.h[1],self.h[2],self.h[3],self.h[4],self.h[5],self.h[6],self.h[7]);for i in 0..64{let s1=e.rotate_right(6)^e.rotate_right(11)^e.rotate_right(25);let ch=(e&f)^(!e&g);let t1=h.wrapping_add(s1).wrapping_add(ch).wrapping_add(K[i]).wrapping_add(w[i]);let s0=a.rotate_right(2)^a.rotate_right(13)^a.rotate_right(22);let maj=(a&b)^(a&c)^(b&c);let t2=s0.wrapping_add(maj);h=g;g=f;f=e;e=d.wrapping_add(t1);d=c;c=b;b=a;a=t1.wrapping_add(t2);}for(i,v)in[a,b,c,d,e,f,g,h].iter().enumerate(){self.h[i]=self.h[i].wrapping_add(*v);}}
}
