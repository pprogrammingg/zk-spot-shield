declare module "circomlibjs" {
  export function buildPoseidon(): Promise<
    ((inputs: unknown[]) => unknown) & {
      F: {
        e: (x: bigint | string | number) => unknown;
        toObject: (x: unknown) => bigint;
      };
    }
  >;
}
