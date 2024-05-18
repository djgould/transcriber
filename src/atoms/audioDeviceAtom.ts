import { atomWithStorage } from "jotai/utils";

export const selectedAudioDeviceAtom = atomWithStorage<string | undefined>(
  "selectedAudioDevice",
  undefined,
  undefined,
  {
    getOnInit: true,
  }
);
