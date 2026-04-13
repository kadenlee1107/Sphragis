// Bat_OS — Ethernet Frame Handler

pub const ETH_HDR_SIZE: usize = 14;
pub const ETHERTYPE_ARP: u16 = 0x0806;
pub const ETHERTYPE_IPV4: u16 = 0x0800;

pub const BROADCAST: [u8; 6] = [0xff; 6];

pub struct EthFrame<'a> {
    pub dst: [u8; 6],
    pub src: [u8; 6],
    pub ethertype: u16,
    pub payload: &'a [u8],
}

impl<'a> EthFrame<'a> {
    pub fn parse(data: &'a [u8]) -> Option<Self> {
        if data.len() < ETH_HDR_SIZE {
            return None;
        }
        let mut dst = [0u8; 6];
        let mut src = [0u8; 6];
        dst.copy_from_slice(&data[0..6]);
        src.copy_from_slice(&data[6..12]);
        let ethertype = u16::from_be_bytes([data[12], data[13]]);
        Some(Self { dst, src, ethertype, payload: &data[ETH_HDR_SIZE..] })
    }

    pub fn build(dst: &[u8; 6], src: &[u8; 6], ethertype: u16, payload: &[u8], buf: &mut [u8]) -> usize {
        buf[0..6].copy_from_slice(dst);
        buf[6..12].copy_from_slice(src);
        buf[12..14].copy_from_slice(&ethertype.to_be_bytes());
        let len = payload.len().min(buf.len() - ETH_HDR_SIZE);
        buf[ETH_HDR_SIZE..ETH_HDR_SIZE + len].copy_from_slice(&payload[..len]);
        ETH_HDR_SIZE + len
    }
}
