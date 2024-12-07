const { ops } = globalThis.Deno.core;

function myOp2() {
  return ops.custom_op_my_op2();
}

globalThis.MyExt2 = { myOp2 };
