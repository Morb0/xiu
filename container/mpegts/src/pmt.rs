use super::crc32;
use super::define::epat_pid;
use super::define::epsi_stream_type;
use super::errors::MpegTsError;
use super::pes;
use byteorder::BigEndian;
use bytes::BytesMut;
use networkio::bytes_writer::BytesWriter;
#[derive(Debug, Clone)]
pub struct Pmt {
    pub pid: u8,
    pub program_number: u16,
    pub version_number: u8,       //5 bits
    pub continuity_counter: u8,   //4i bits
    pub pcr_pid: u16,             //13 bits
    pub program_info_length: u16, //12 bits

    pub program_info: BytesMut,
    pub provider: [char; 64],
    pub name: [char; 64],
    pub stream_count: u8,
    pub streams: [pes::Pes; 4],
}

impl Pmt {
    //p49
    pub fn write(&mut self) {}
}

pub struct PmtWriter {
    pub bytes_writer: BytesWriter,
}

impl PmtWriter {
    pub fn new() -> Self {
        Self {
            bytes_writer: BytesWriter::new(),
        }
    }

    pub fn write(&mut self, pmt: &Pmt) -> Result<(), MpegTsError> {
        /*table id*/
        self.bytes_writer.write_u8(epat_pid::PAT_TID_PMS)?;

        let mut tmp_bytes_writer = BytesWriter::new();
        /*program_number*/
        tmp_bytes_writer.write_u16::<BigEndian>(pmt.program_number)?;
        /*version_number*/
        tmp_bytes_writer.write_u8(0xC1 | (pmt.version_number << 1))?;
        /*section_number*/
        tmp_bytes_writer.write_u8(0x00)?;
        /*last_section_number*/
        tmp_bytes_writer.write_u8(0x00)?;
        /*PID*/
        tmp_bytes_writer.write_u16::<BigEndian>(0xE000 | pmt.pcr_pid)?;
        /*program_info_length*/
        tmp_bytes_writer.write_u16::<BigEndian>(0xF000 | pmt.program_info_length)?;

        if pmt.program_info_length > 0 && pmt.program_info_length < 0x400 {
            tmp_bytes_writer.write(&pmt.program_info[..])?;
        }

        for stream in &pmt.streams {
            /*stream_type*/
            let stream_type: u8;
            if stream.codec_id == epsi_stream_type::PSI_STREAM_AUDIO_OPUS {
                stream_type = epsi_stream_type::PSI_STREAM_PRIVATE_DATA;
            } else {
                stream_type = stream.codec_id;
            }
            tmp_bytes_writer.write_u8(stream_type)?;
            /*elementary_PID*/
            tmp_bytes_writer.write_u16::<BigEndian>(0xE000 | stream.pid)?;
            /*ES_info_length*/
            tmp_bytes_writer.write_u16::<BigEndian>(0xF000)?;
        }

        /*section_length*/
        self.bytes_writer
            .write_u16::<BigEndian>(0xB000 | (tmp_bytes_writer.len() as u16))?;

        self.bytes_writer
            .write(&tmp_bytes_writer.extract_current_bytes()[..])?;

        /*crc32*/
        let crc32_value = crc32::gen_crc32(0xffffffff, self.bytes_writer.extract_current_bytes());
        self.bytes_writer.write_u32::<BigEndian>(crc32_value)?;

        Ok(())
    }

    pub fn write_descriptor(&mut self) -> Result<(), MpegTsError> {
        Ok(())
    }
}
