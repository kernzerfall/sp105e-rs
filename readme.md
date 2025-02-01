<div align=center><h1>sp105e-rs</h1></div>

<p></p>

<div align=center>
<p>
  A complete¹ reverse-engineered² Rust implementation of the protocol
  used by the Bluetooth LE variant of the SP105E LED controller.
</p>
</div>

<p></p>
<p>
</p>

## Library

The library can be found under `sp105e`.

- [src/commands.rs](sp105e/src/commands.rs)
  contains the command definitions and buffer building logic,
  as well as the (partial) decoder for the status message.
- [src/client.rs](sp105e/src/client.rs)
  contains a simple BLE abstraction layer using BlueR (BlueZ).
  Obviously supported only on Linux. Feature: `client`.

## CLI

`sp105e-cli` provides a simple CLI using `sp105e::client`.
Only for Linux (due to the BlueR/BlueZ dependency).

For usage help try `sp105e-cli --help`.

<hr>

<p></p>

¹The implementation is *complete* as in *can do anything the app can do*.

²Special thanks goes to
 <a href="https://github.com/anetczuk/BluetoothGattMitm">BluetoothGattMitm</a>.
