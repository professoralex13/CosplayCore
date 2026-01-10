import 'package:flutter/material.dart';
import 'package:flutter_blue_classic/flutter_blue_classic.dart';

class DeviceView extends StatelessWidget {
  const DeviceView(this.device, {super.key});

  final BluetoothConnection device;

  @override
  Widget build(BuildContext context) {
    return Column(children: [Text('Connected to Device')]);
  }
}
