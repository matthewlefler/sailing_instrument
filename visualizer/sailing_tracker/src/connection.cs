using System;
using System.Collections.Generic;
using System.Numerics;
using System.Security.AccessControl;
using Microsoft.Xna.Framework.Graphics;

namespace connection;

public class TrackerData
{
    public double position_x = 0.0;
    public double position_y = 0.0;
    public double position_z = 0.0;

    public double gyro_x = 0.0;
    public double gyro_y = 0.0;
    public double gyro_z = 0.0;

    public TrackerData(string message)
    {
        var sections = message.Split(" ");
        Double.TryParse(sections[0], out position_x);
        Double.TryParse(sections[1], out position_y);
        Double.TryParse(sections[2], out position_z);

        Double.TryParse(sections[3], out gyro_x);
        Double.TryParse(sections[4], out gyro_y);
        Double.TryParse(sections[5], out gyro_z);
    }

    public Vector3 get_position()
    {
        return new Vector3((float) position_x, (float) position_y, (float) position_z);
    }
}

public interface Connection
{
    public List<TrackerData> read();
}