import 'package:cosplay_core_app/bluetooth_state.dart';
import 'package:cosplay_core_app/device_view.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_blue_classic/flutter_blue_classic.dart';
import 'package:provider/provider.dart';
import 'package:provider/single_child_widget.dart';

List<SingleChildWidget> get providers {
  return [ChangeNotifierProvider(create: (context) => BluetoothState())];
}

void main() {
  runApp(MultiProvider(providers: providers, child: const MainApp()));
}

class MainApp extends StatelessWidget {
  const MainApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      home: Scaffold(
        appBar: AppBar(title: const Text("CosplayCore App")),
        body: DeviceList(),
      ),
    );
  }
}

class DeviceList extends StatefulWidget {
  const DeviceList({super.key});

  @override
  State<DeviceList> createState() => _DeviceListState();
}

class _DeviceListState extends State<DeviceList> {
  List<BluetoothDevice> _devices = [];

  @override
  Widget build(BuildContext context) {
    BluetoothState bluetoothState = context.watch<BluetoothState>();

    if (bluetoothState.getAdapterState() != BluetoothAdapterState.on) {
      return Placeholder();
    }

    var connectedDevice = bluetoothState.getConnectedDevice();

    if (connectedDevice != null) {
      return DeviceView(connectedDevice);
    }

    return RefreshIndicator(
      child: ListView.builder(
        itemCount: _devices.length,
        itemBuilder: (context, index) {
          var device = _devices[index];

          return ListTile(
            title: Text(device.name ?? 'Unkown'),
            onTap: () async {
              try {
                await bluetoothState.connectToDevice(device);
              } on PlatformException {
                final snackBar = SnackBar(
                  content: Text('Error connecting to device: "${device.name}"'),
                  duration: Duration(seconds: 2),
                );

                ScaffoldMessenger.of(context).showSnackBar(snackBar);
              }
            },
          );
        },
      ),
      onRefresh: () async {
        var devices = await bluetoothState.getDevices();

        setState(() {
          _devices = devices;
        });
      },
    );
  }
}
