import { atomWithStorage } from "jotai/utils";

export const selectedAudioInputDeviceAtom = atomWithStorage<string | undefined>(
  "selectedAudioInputDevice",
  undefined,
  undefined,
  {
    getOnInit: true,
  }
);

export const selectedAudioOutputDeviceAtom = atomWithStorage<
  string | undefined
>("selectedAudioOutputDevice", undefined, undefined, { getOnInit: true });
