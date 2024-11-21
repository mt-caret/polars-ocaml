module C = Configurator.V1

let () =
  C.main ~name:"polars-ocaml" (fun c ->
    let c_library_flags =
      match C.ocaml_config_var c "system" with
      | Some "macosx" -> [ "-framework"; "CoreFoundation"; "-framework"; "IOKit" ]
      | _ -> []
    in
    C.Flags.write_sexp "c_library_flags.sexp" c_library_flags)
;;
