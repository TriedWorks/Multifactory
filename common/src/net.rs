pub mod io {
    use tokio::prelude::*;
    use tokio::net::TcpStream;
    use tokio::fs::File;
    use std::process::exit;

    pub async fn send_string(send: String, stream: &mut TcpStream) {
        // Send the string size and the string to the Client
        let send_string_size = send.len();
        stream.write_u8(send_string_size as u8).await;
        stream.write(send.as_bytes()).await;
    }

    pub async fn send_file(filename: String, stream: &mut TcpStream) {
        // Try to open the open the file to be sent
        let mut open_file_result = File::open(filename).await;

        // If the file can't be opened, don't send anything
        if open_file_result.is_err() {
            stream.write_u8(0).await;
            return;
        }

        // Get the filesize
        let mut file = open_file_result.unwrap();
        let file_size = file.metadata().len();

        // Create the file content buffer with the according size
        let mut file_contents = vec![0 as u8; file_size];

        // Write the file contents into the file content buffer and send
        //  nothing if this does not occur
        if file.read_exact(&mut file_contents).await.is_err() {
            stream.write_u8(0).await;
            return;
        }

        // Only now send the file size, since we can be sure that the file
        //  was read correctly
        stream.write_u64(file_size as u64).await;

        // And of course, send the file contents buffer
        stream.write_all(file_contents.as_slice()).await;
    }

    pub async fn receive_file(filename: String, stream: &mut TcpStream) {
        let new_file_size_result = stream.read_u64().await;

        if new_file_size_result.is_err() {
            println!("Failed to receive File size!");
            exit(1);
        }

        let new_file_size = new_file_size_result.unwrap();

        if new_file_size == 0 {
            println!("File size is 0 (File does not exist)!");
            exit(1);
        }

        let create_file_result = File.create(filename).await;

        if create_file_result.is_err() {
            println!("Failed to create File!");
            exit(1);
        }

        let mut file: File = create_file_result.unwrap();
        file.set_len(new_file_size).await;

        let mut file_buffer = vec![0 as u8; new_file_size as usize];

        if stream.read_exact(&mut file_buffer).await.is_err() {
            println!("Failed to receive File contents!");
            exit(1);
        }

        if file.write_all(file_buffer.as_slice()).await.is_err() {
            println!("Failed to write contents to File!");
            exit(1);
        }
    }

    pub async fn receive_string(stream: &mut TcpStream) -> String {
        let receive_string_length_result = stream.read_u8().await;

        if receive_string_length_result.is_err() {
            println!("Failed to receive String length!");
            exit(1);
        }

        let string_length = receive_string_length_result.unwrap();

        let mut string_buffer = vec![0 as u8; string_length as usize];

        if stream.read_exact(&mut string_buffer).await.is_err() {
            println!("Failed to receive String!");
            exit(1);
        }

        let mut create_string_from_bytes_result = String::from_utf8(string_buffer);

        if create_string_from_bytes_result.is_err() {
            println!("Failed to create String from byte sequence!");
            exit(1);
        }

        return create_string_from_bytes_result.unwrap();
    }
}