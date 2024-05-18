import { useAudioDevicesQuery } from "@/hooks/useMediaDevices";
import React from "react";

const initialState: {
  audioDevice: undefined | string;
  setAudioDevice: (value: string) => void;
} = {
  audioDevice: undefined,
  setAudioDevice: (value: string) => {},
};

export const AudioDeviceContext = React.createContext(initialState);

export const AudioDeviceContextProvider = ({
  children,
}: {
  children: React.ReactNode;
}) => {
  const mediaDevices = useAudioDevicesQuery();
  const [audioDevice, setAudioDevice] = React.useState<string | undefined>();

  const selectedDeviceOrDefault = audioDevice || mediaDevices.data?.[0];

  return (
    <AudioDeviceContext.Provider
      value={{ audioDevice: selectedDeviceOrDefault, setAudioDevice }}
    >
      {children}
    </AudioDeviceContext.Provider>
  );
};

export const useAudioDevice = () => React.useContext(AudioDeviceContext);
