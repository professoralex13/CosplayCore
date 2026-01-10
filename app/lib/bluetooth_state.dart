import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_blue_classic/flutter_blue_classic.dart';

class BluetoothState extends ChangeNotifier {
  final _bluetoothPlugin = FlutterBlueClassic();

  late StreamSubscription _adapterStateSubscription;

  BluetoothConnection? _connection;

  BluetoothAdapterState _adapterState = BluetoothAdapterState.unknown;

  void _onAdapterState(BluetoothAdapterState current) {
    _adapterState = current;

    notifyListeners();
  }

  BluetoothState() {
    _bluetoothPlugin.adapterStateNow.then(_onAdapterState);

    _adapterStateSubscription = _bluetoothPlugin.adapterState.listen(
      _onAdapterState,
    );

    _bluetoothPlugin.turnOn();
  }

  BluetoothAdapterState getAdapterState() {
    return _adapterState;
  }

  Future<List<BluetoothDevice>> getDevices() async {
    final devices = await _bluetoothPlugin.bondedDevices;

    if (devices == null) {
      return [];
    }

    return devices.where((device) {
      return device.name?.startsWith("CosplayCore") ?? false;
    }).toList();
  }

  BluetoothConnection? getConnectedDevice() {
    return _connection;
  }

  Future<void> connectToDevice(BluetoothDevice device) async {
    var connection = await _bluetoothPlugin.connect(device.address);

    if (connection != null) {
      _connection = connection;

      notifyListeners();
    }
  }

  @override
  void dispose() {
    super.dispose();

    _adapterStateSubscription.cancel();
  }
}
