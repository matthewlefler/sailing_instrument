namespace connection;

using System;
using System.Collections.Generic;
using System.IO.Ports;

public class Esp32SerialConnection : connection.Connection
{
    private SerialPort port = new SerialPort("/dev/ttyUSB0", 115200, Parity.None, 8, StopBits.One);

    public Esp32SerialConnection()
    {
        port.Open();
    }

    public List<TrackerData> read()
    {
        var messages = new List<TrackerData>();

        if(port.BytesToRead <= 0)
        {
            return messages;
        }

        try
        {
            string message = port.ReadLine();

            messages.Add(new TrackerData(message));
        } 
        catch
        {
            return messages;
        }

        return messages;
    }
}