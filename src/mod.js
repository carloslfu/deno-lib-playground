const { ops } = globalThis.Deno.core;

function documentDir() {
  return ops.custom_op_document_dir();
}

globalThis.RuntimeExtension = { documentDir };
