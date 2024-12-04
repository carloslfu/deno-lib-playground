import { op_document_dir } from "ext:core/ops";

function documentDir() {
  return op_document_dir();
}

globalThis.RuntimeExtension = { documentDir };
