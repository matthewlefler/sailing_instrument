namespace connection;

using System;
using System.IO.Ports;

public class Esp32SerialConnection : Connection
{
    private SerialPort port = new SerialPort("COM1", 9600, Parity.None, 8, StopBits.One);

    public Esp32SerialConnection()
    {
        
    }
}