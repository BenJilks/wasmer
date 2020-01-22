initSidebarItems({"constant":[["ALL_RIGHTS","all the rights enabled"],["MAX_SYMLINKS","A completely aribtrary \"big enough\" number used as the upper limit for the number of symlinks that can be traversed when resolving a path"],["VIRTUAL_ROOT_FD","the fd value of the virtual root"]],"enum":[["Kind","The core of the filesystem abstraction.  Includes directories, files, and symlinks."],["PollEvent",""],["WasiFsError","Error type for external users"],["WasiStateCreationError","Error type returned when bad data is given to [`WasiStateBuilder`]."]],"fn":[["get_wasi_state","Get WasiState from a Ctx This function is unsafe because it must be called on a WASI Ctx"],["host_file_type_to_wasi_file_type",""],["iterate_poll_events",""]],"struct":[["Fd",""],["HostFile","A thin wrapper around `std::fs::File`"],["Inode","An index (and generation) into an `Arena`."],["InodeVal","A file that Wasi knows about that may or may not be open"],["PollEventBuilder",""],["PollEventIter",""],["PreopenDirBuilder","Builder for preopened directories."],["Stderr","A wrapper type around Stderr that implements `WasiFile` and `Serialize` + `Deserialize`."],["Stdin","A wrapper type around Stdin that implements `WasiFile` and `Serialize` + `Deserialize`."],["Stdout","A wrapper type around Stdout that implements `WasiFile` and `Serialize` + `Deserialize`."],["WasiFs","Warning, modifying these fields directly may cause invariants to break and should be considered unsafe.  These fields may be made private in a future release"],["WasiState","Top level data type containing all* the state with which WASI can interact."],["WasiStateBuilder","Convenient builder API for configuring WASI via [`WasiState`]."]],"trait":[["WasiFile","This trait relies on your file closing when it goes out of scope via `Drop`"],["WasiPath",""]],"type":[["PollEventSet",""]]});