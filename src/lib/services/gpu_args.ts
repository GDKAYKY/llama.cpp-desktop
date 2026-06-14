const MAIN_GPU_FLAGS = new Set(["-mg", "--main-gpu"]);

function isMainGpuAssignment(arg: string) {
  return arg.startsWith("--main-gpu=");
}

export function withMainGpuArg(extraArgs: string[] | undefined, gpuDevice: string | undefined) {
  const args = (extraArgs ?? []).filter((arg) => arg.trim().length > 0);
  const device = gpuDevice?.trim();

  if (!device) {
    return args;
  }

  const hasMainGpuArg = args.some(
    (arg, index) =>
      MAIN_GPU_FLAGS.has(arg) ||
      isMainGpuAssignment(arg) ||
      (index > 0 && MAIN_GPU_FLAGS.has(args[index - 1])),
  );

  if (hasMainGpuArg) {
    return args;
  }

  return [...args, "--main-gpu", device];
}
