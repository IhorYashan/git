pub mod git {
    mod zlib;

    extern crate hex;

    use std::collections::HashMap;

    use std::fs;
    use std::fs::File;
    use std::io;
    use std::io::Read;

    const HASH_BYTES: usize = 20;

    // --- Public functions --- //
    pub fn do_git_init() {
        create_dir("");
    }

    pub fn read_git_object(blob_file: &str) -> io::Result<()> {
        let mut file_content = Vec::new();

        let hash_path = &blob_file[..2];
        let hash_file = &blob_file[2..];

        let object_path = format!(".git/objects/{}/{:}", hash_path, hash_file);
        let mut object_file = File::open(&object_path)?;

        object_file.read_to_end(&mut file_content)?;

        let compressed_data = &file_content[..];
        let buffer = zlib::decode_data(compressed_data).0;

        #[allow(unsafe_code)]
        let string_buffer = unsafe { String::from_utf8_unchecked(buffer) };

        print!("{}", &string_buffer[8..]);

        Ok(())
    }

    pub fn write_git_object(
        content_file: Vec<u8>,
        file_type: &str,
        target_dir: &str,
    ) -> Result<String, io::Error> {
        let content_str = String::from_utf8(content_file.clone())
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;

        let header_blob = format!("{} {}\x00", file_type, content_file.len());
        let data_to_compress = format!("{}{}", header_blob, content_str);

        let (hash_blob_file, compressed_data) = zlib::encode_data(data_to_compress);

        let hash_dir = &hash_blob_file[..2];
        let hash_file = &hash_blob_file[2..];

        let sub_hash_path_dir = if target_dir != "./" {
            format!("{}/.git/objects/{}/", target_dir, hash_dir)
        } else {
            format!(".git/objects/{}/", hash_dir)
        };

        let full_hash_path_dir = format!("{}{}", sub_hash_path_dir, hash_file);

        fs::create_dir_all(&sub_hash_path_dir)?;
        fs::write(&full_hash_path_dir, compressed_data)?;

        Ok(hash_blob_file)
    }

    pub fn read_tree_object(sha_tree: String) -> Result<(), io::Error> {
        let hash_dir = &sha_tree[..2];
        let hash_tree_object = &sha_tree[2..];

        let full_path = format!(".git/objects/{}/{}", hash_dir, hash_tree_object);
        let mut file = File::open(full_path)?;

        let mut file_content = Vec::new();
        file.read_to_end(&mut file_content)?;

        let decompressed_data = zlib::decode_data(&file_content).0;

        let string_buffer = String::from_utf8_lossy(&decompressed_data);

        let parts: Vec<&str> = string_buffer.split('\x00').skip(1).collect();

        for part in parts {
            if let Some(word) = part.split(' ').nth(1) {
                println!("{}", word);
            }
        }

        Ok(())
    }
    pub fn write_tree_object(file_path: &str) -> Result<String, io::Error> {
        let mut sha_out = String::new();
        let entries = fs::read_dir(file_path)?;

        for entry in entries {
            let entry = entry?;
            let file_type = entry.file_type()?;
            let file_name = entry.file_name().to_string_lossy().into_owned();

            if file_name == ".git" {
                continue;
            }

            let mode = if file_type.is_dir() {
                "40000"
            } else {
                "100644"
            };

            let sha_file: String;
            if file_type.is_dir() {
                let sub_directory = entry.path();
                let sha_file1 = write_tree_object(sub_directory.to_str().unwrap())?;
                sha_file = hex::decode(&sha_file1)
                    .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?
                    .iter()
                    .map(|b| format!("{:02x}", b))
                    .collect();
            } else {
                let mut content_file = Vec::new();
                fs::File::open(&entry.path())?.read_to_end(&mut content_file)?;
                let sha_file1 = write_git_object(content_file, "blob", "./")?;
                sha_file = sha_file1;
            }

            sha_out += &format!("{} {}{}\x00", mode, file_name, sha_file);
        }

        let res_sha = write_git_object(sha_out.into_bytes(), "tree", "./")?;
        Ok(res_sha)
    }

    // complete with consts
    pub fn do_commit(
        tree_sha: String,
        commit_sha: String,
        message: String,
    ) -> Result<String, io::Error> {
        let content_commit = format!(
            "tree {}\nparent {}\nauthor ScotChacon <schacon@gmail.com> 1243040974 -0700\ncommitter ScotChacon <schacon@gmail.com> 1243040974 -0700\n\n{}\n",
            tree_sha, commit_sha, message
        );

        let sha_commit = write_git_object(content_commit.into_bytes(), "commit", "./")?;
        Ok(sha_commit)
    }

    pub fn clone_repo(dir_name: String, link: String) -> Result<(), io::Error> {
        create_dir(&(dir_name.to_owned() + "/"));

        let dir_obj = dir_name.to_owned() + "/.git/objects/";

        let post_url = link.clone() + &"/git-upload-pack".to_string();
        let link = format!("{}/info/refs?service=git-upload-pack", link);

        let body = reqwest::blocking::get(link.clone())
            .unwrap()
            .text()
            .unwrap();

        let sha_refs = extract_commit_hash(&body);

        let sha_refs = &sha_refs[..40];

        let body = format!("0032want {}\n00000009done\n", &sha_refs);
        println!("post_url : {}, body : {} ", post_url, body);
        let data_from_git = get_data_form_git(post_url.clone(), body);

        let git_data_size = data_from_git.len() - HASH_BYTES;
        println!("git_data_size : {}", git_data_size);
        let entries_bytes = &data_from_git[16..HASH_BYTES];
        println!("entries_bytes : {:?}", entries_bytes);
        let num = u32::from_be_bytes(entries_bytes.try_into().unwrap());
        println!("num: {:?}", num);
        let data_bytes: Vec<u8> = data_from_git[HASH_BYTES..git_data_size].try_into().unwrap();

        let mut objects = HashMap::new();
        let mut seek = 0;
        let mut obj_counter = 0;

        while obj_counter != num {
            obj_counter += 1;
            let first = data_bytes[seek];
            let mut obj_type: usize = ((first & 112) >> 4).into();
            println!("obj_type: {:?}", obj_type);
            while data_bytes[seek] > 128 {
                seek += 1;
            }
            seek += 1;
            // println!("seek : {:?}", seek);
            let data_type = [
                "",
                "commit",
                "tree",
                "blob",
                "",
                "tag",
                "ofs_delta",
                "refs_delta",
            ];
            if obj_type < 7 {
                println!("{}", obj_counter);

                let (git_data, bytes) = zlib::decode_data(&data_bytes[seek..]);

                #[allow(unsafe_code)]
                let string_buffer = unsafe { String::from_utf8_unchecked(git_data) };

                let hash_obj = write_git_object(
                    string_buffer.clone().into_bytes(),
                    data_type[obj_type],
                    &dir_name,
                )?;

                objects.insert(hash_obj, (string_buffer, obj_type));

                seek += bytes;
            } else {
                let obj_data_bytes = &data_bytes[seek..seek + HASH_BYTES];

                let obj_data_bytes = hex::encode(obj_data_bytes);
                let (base, elem_num) = objects[&obj_data_bytes].to_owned();
                seek += HASH_BYTES;

                let (git_data, bytes) = zlib::decode_data(&data_bytes[seek..]);

                #[allow(unsafe_code)]
                let string_buffer = unsafe { String::from_utf8_unchecked(git_data) };

                let content = apply_delta(&string_buffer.as_bytes(), &base.as_bytes());

                obj_type = elem_num;

                let hash_obj =
                    write_git_object(content.clone().into(), data_type[obj_type], &dir_name)?;

                objects.insert(hash_obj, (content, obj_type));

                seek += bytes;
            }
        }

        let git_path_pack =
            dir_name.to_owned() + &format!("/.git/objects/{}/{}", &sha_refs[..2], &sha_refs[2..]);

        println!("git_path_pack : {:?}", git_path_pack);

        let git_data = fs::read(git_path_pack).unwrap();

        let (delta, _bytes) = zlib::decode_data(&git_data.to_vec());

        #[allow(unsafe_code)]
        let string_buffer_delta = unsafe { String::from_utf8_unchecked(delta) };

        let data = string_buffer_delta.split("\n").next().unwrap().split(" ");

        let sha_obj = data.clone().nth(data.count() - 1).unwrap();
        println!("sha_obj: {:?}", &sha_obj);

        checkout(&sha_obj, &dir_name, &dir_obj);

        Ok(())
    }

    // --- Private functions --- //

    fn create_dir(dir_name: &str) {
        if dir_name != "" {
            fs::create_dir(dir_name).unwrap();
        }

        fs::create_dir(dir_name.to_owned() + ".git").unwrap();
        println!("{}", dir_name.to_owned() + ".git");
        fs::create_dir(dir_name.to_owned() + ".git/objects/").unwrap();
        fs::create_dir(dir_name.to_owned() + ".git/refs").unwrap();
        fs::write(
            dir_name.to_owned() + ".git/HEAD",
            "ref: refs/heads/master\n",
        )
        .unwrap();
    }

    fn checkout(sha: &str, file_path: &str, dir_name: &str) {
        fs::create_dir_all(file_path).unwrap();

        let git_data = fs::read(format!("{}/{}/{}", dir_name, &sha[..2], &sha[2..])).unwrap();
        let (decoded_data, _bytes) = zlib::decode_data(&git_data[..]);

        let pos = decoded_data
            .iter()
            .position(|&r| r == '\x00' as u8)
            .unwrap();
        let mut tree = &decoded_data[pos + 1..];

        let mut entries = Vec::new();

        while !tree.is_empty() {
            let pos = tree.iter().position(|&r| r == '\x00' as u8).unwrap();
            let mode_name = &tree[..pos];
            let mut mode_name = mode_name.splitn(2, |&num| num == ' ' as u8);
            let mode = mode_name.next().unwrap();
            let name = mode_name.next().unwrap();
            tree = &tree[pos + 1..];

            let sha = &tree[..HASH_BYTES];
            tree = &tree[HASH_BYTES..];

            let sha = hex::encode(&sha[..]);
            let mode = String::from_utf8_lossy(mode).into_owned();
            let name = String::from_utf8_lossy(name).into_owned();

            entries.push((mode, name, sha));
        }

        for entry in entries {
            if entry.0 == "40000" {
                let subdir_path = format!("{}/{}", file_path, entry.1);
                checkout(&entry.2, &subdir_path, dir_name);
            } else {
                let blob_sha = &entry.2;
                let curr_dir = format!("{}/{}/{}", dir_name, &blob_sha[..2], &blob_sha[2..]);
                let git_data = fs::read(curr_dir).unwrap();
                let (decoded_data, _bytes) = zlib::decode_data(&git_data[..]);

                let pos = decoded_data
                    .iter()
                    .position(|&r| r == '\x00' as u8)
                    .unwrap();
                let content = &decoded_data[pos + 1..];

                let file_path = format!("{}/{}", file_path, entry.1);
                fs::write(&file_path, content).unwrap();
            }
        }
    }

    fn apply_delta(delta: &[u8], base: &[u8]) -> String {
        let mut seek: usize = 0;
        let mut content = String::new();

        while seek < delta.len() {
            let instr_byte = delta[seek];
            seek += 1;

            if instr_byte >= 128 {
                let offset_key = instr_byte & 0b00001111;
                let mut offset_bytes: [u8; 8] = [0; 8];

                for n in 0..8 {
                    if (offset_key >> n) & 1 == 1 {
                        offset_bytes[n] = delta[seek];
                        seek += 1;
                    }
                }
                let offset = usize::from_le_bytes(offset_bytes);

                let len_key = (instr_byte & 0b01110000) >> 4;
                let mut len_bytes: [u8; 8] = [0; 8];

                for n in 0..8 {
                    if (len_key >> n) & 1 == 1 {
                        len_bytes[n] = delta[seek];
                        seek += 1;
                    }
                }
                let len_int = usize::from_le_bytes(len_bytes);

                content += &String::from_utf8_lossy(&base[offset..(offset + len_int)]);
            } else {
                let num_bytes = instr_byte as usize;

                content += &String::from_utf8_lossy(&delta[seek..(seek + num_bytes)]);
                seek += num_bytes;
            }
        }
        content
    }

    fn get_data_form_git(link: String, body: String) -> bytes::Bytes {
        let client = reqwest::blocking::Client::new();
        let client_req = client
            .post(link)
            .header("content-type", "application/x-git-upload-pack-request")
            .body(body);

        let response_data = client_req.send().unwrap();
        let response_data = response_data.bytes().unwrap();

        response_data
    }
    //
    fn extract_commit_hash(response: &str) -> &str {
        let index = match response.find("refs/heads/master\n0000") {
            Some(index) => index,
            None => panic!("can`t find sha !"),
        };
        let sha_refs = &response[index - 41..index];

        sha_refs
    }
}
