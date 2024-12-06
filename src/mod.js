// import { document_dir } from "ext:runtime_extension/mod.js";

function documentDir() {
  // return document_dir();
  return "--called---";
}

globalThis.RuntimeExtension = { documentDir };
