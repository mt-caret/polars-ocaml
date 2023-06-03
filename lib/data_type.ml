module Time_unit = struct
  type t =
    | Nanoseconds
    | Microseconds
    | Milliseconds
end

type t =
  | Boolean
  | UInt8
  | UInt16
  | UInt32
  | UInt64
  | Int8
  | Int16
  | Int32
  | Int64
  | Float32
  | Float64
  | Utf8
  | Binary
  | Date
  | Datetime of Time_unit.t
  | Time
  | Null
  | Unknown
