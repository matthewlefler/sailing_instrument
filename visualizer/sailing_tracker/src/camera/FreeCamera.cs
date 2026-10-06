using Microsoft.Xna.Framework;
using Microsoft.Xna.Framework.Input;

public class FreeCamera
{
    public Vector3 Position { get; set; }
    public float Yaw { get; set; }
    public float Pitch { get; set; }
    public float Speed { get; set; } = 5f;
    public float Sensitivity { get; set; } = 0.005f;

    private Matrix _view;
    private Matrix _projection;

    public FreeCamera(float aspectRatio, Vector3 startPosition)
    {
        Position = startPosition;
        _projection = Matrix.CreatePerspectiveFieldOfView(
            MathHelper.ToRadians(60f), aspectRatio, 0.1f, 1000f);
    }

    public void Update(GameTime gameTime, KeyboardState kb, MouseState mouse,
                       Point screenCentre)
    {
        float dt = (float)gameTime.ElapsedGameTime.TotalSeconds;

        // Mouse look
        float dx = mouse.X - screenCentre.X;
        float dy = mouse.Y - screenCentre.Y;
        Yaw -= dx * Sensitivity;
        Pitch -= dy * Sensitivity;
        Pitch = MathHelper.Clamp(Pitch,
            MathHelper.ToRadians(-89f), MathHelper.ToRadians(89f));

        // Direction vectors
        var rotation = Matrix.CreateRotationX(Pitch) *
                       Matrix.CreateRotationY(Yaw);
        var forward = Vector3.Transform(Vector3.Forward, rotation);
        var right = Vector3.Transform(Vector3.Right, rotation);
        var up = Vector3.Transform(Vector3.Up, rotation);

        // Movement
        if (kb.IsKeyDown(Keys.W)) Position += forward * Speed * dt;
        if (kb.IsKeyDown(Keys.S)) Position -= forward * Speed * dt;
        if (kb.IsKeyDown(Keys.A)) Position -= right * Speed * dt;
        if (kb.IsKeyDown(Keys.D)) Position += right * Speed * dt;
        if (kb.IsKeyDown(Keys.E)) Position += up * Speed * dt;
        if (kb.IsKeyDown(Keys.Q)) Position -= up * Speed * dt;

        _view = Matrix.CreateLookAt(Position, Position + forward, Vector3.Up);
    }

    public Matrix View => _view;
    public Matrix Projection => _projection;
}