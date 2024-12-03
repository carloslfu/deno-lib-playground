import { document_dir } from "ext:core/ops";

function documentDir() {
  return document_dir();
}

globalThis.RuntimeExtension = { documentDir };
