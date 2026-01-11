import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:flutter_blue_classic/flutter_blue_classic.dart';
import 'dart:ffi' as ffi;
import 'package:ffi/ffi.dart';

enum MessageType {
  setInputVolume(0),
  setOutputVolume(1),
  setDACVolume(2);

  const MessageType(this.value);
  final int value;
}

final class VolumeMessage extends ffi.Struct {
  @ffi.Uint8()
  external int volume;
}

class DeviceView extends StatelessWidget {
  const DeviceView(this.device, {super.key});

  final BluetoothConnection device;

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        Text('Connected to Device'),
        IntSlider(
          min: 0,
          max: 63,
          label: "Input Volume",
          onChanged: (int value) {
            final ffi.Pointer<VolumeMessage> messagePointer =
                calloc<VolumeMessage>();

            messagePointer.ref.volume = value;

            final int structSize = ffi.sizeOf<VolumeMessage>();

            var builder = BytesBuilder();

            builder.addByte(MessageType.setInputVolume.value);
            builder.add(
              messagePointer.cast<ffi.Uint8>().asTypedList(structSize),
            );

            device.output.add(builder.toBytes());
          },
        ),
        IntSlider(
          min: 0,
          max: 127,
          label: "Output Volume",
          onChanged: (int value) {
            final ffi.Pointer<VolumeMessage> messagePointer =
                calloc<VolumeMessage>();

            messagePointer.ref.volume = value;

            final int structSize = ffi.sizeOf<VolumeMessage>();

            var builder = BytesBuilder();

            builder.addByte(MessageType.setOutputVolume.value);
            builder.add(
              messagePointer.cast<ffi.Uint8>().asTypedList(structSize),
            );

            device.output.add(builder.toBytes());
          },
        ),
        IntSlider(
          min: 0,
          max: 255,
          label: "DAC Volume",
          onChanged: (int value) {
            final ffi.Pointer<VolumeMessage> messagePointer =
                calloc<VolumeMessage>();

            messagePointer.ref.volume = value;

            final int structSize = ffi.sizeOf<VolumeMessage>();

            var builder = BytesBuilder();

            builder.addByte(MessageType.setDACVolume.value);
            builder.add(
              messagePointer.cast<ffi.Uint8>().asTypedList(structSize),
            );

            device.output.add(builder.toBytes());
          },
        ),
      ],
    );
  }
}

class IntSlider extends StatefulWidget {
  const IntSlider({
    required this.min,
    required this.max,
    this.onChanged,
    this.label,
    super.key,
  });

  final int min;
  final int max;
  final String? label;

  final ValueChanged<int>? onChanged;

  @override
  State<IntSlider> createState() => _IntSliderState();
}

class _IntSliderState extends State<IntSlider> {
  late int value = widget.min;

  @override
  Widget build(BuildContext context) {
    return Slider(
      value: value.toDouble(),
      divisions: widget.max - widget.min,
      onChanged: (double value) => {
        widget.onChanged?.call(value.toInt()),
        setState(() {
          this.value = value.toInt();
        }),
      },
      min: widget.min.toDouble(),
      max: widget.max.toDouble(),
      label: widget.label,
    );
  }
}
