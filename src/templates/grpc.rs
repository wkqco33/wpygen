use crate::models::ProjectSpec;

pub(crate) fn build_proto(spec: &ProjectSpec) -> String {
    format!(
        r#"syntax = "proto3";

package {package_name};

service Greeter {{
  rpc SayHello(HelloRequest) returns (HelloReply);
}}

message HelloRequest {{
  string name = 1;
}}

message HelloReply {{
  string message = 1;
}}
"#,
        package_name = spec.package_name,
    )
}

pub(crate) fn build_codegen_script(spec: &ProjectSpec) -> String {
    format!(
        r#"from __future__ import annotations

from pathlib import Path

from grpc_tools import protoc

ROOT_DIR = Path(__file__).resolve().parents[1]
PROTO_DIR = ROOT_DIR / "proto"
OUT_DIR = ROOT_DIR / "src" / "{package_name}" / "grpc"
PROTO_FILE = PROTO_DIR / "{package_name}.proto"
PB2_GRPC_FILE = OUT_DIR / "{package_name}_pb2_grpc.py"


def main() -> int:
    OUT_DIR.mkdir(parents=True, exist_ok=True)
    args = [
        "",
        f"-I{{PROTO_DIR}}",
        f"--python_out={{OUT_DIR}}",
        f"--grpc_python_out={{OUT_DIR}}",
        str(PROTO_FILE),
    ]
    result = int(protoc.main(args))
    if result != 0:
        return result

    generated = PB2_GRPC_FILE.read_text(encoding="utf-8")
    absolute_import = "import {package_name}_pb2 as "
    relative_import = "from . import {package_name}_pb2 as "
    if absolute_import in generated and relative_import not in generated:
        PB2_GRPC_FILE.write_text(
            generated.replace(absolute_import, relative_import, 1),
            encoding="utf-8",
        )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
"#,
        package_name = spec.package_name,
    )
}
