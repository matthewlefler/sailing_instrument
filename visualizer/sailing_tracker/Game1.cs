using System;
using System.Collections.Generic;
using connection;
using Microsoft.VisualBasic;
using Microsoft.Xna.Framework;
using Microsoft.Xna.Framework.Graphics;
using Microsoft.Xna.Framework.Input;

namespace sailing_tracker;

public class Game1 : Game
{
    private GraphicsDeviceManager _graphics;
    private SpriteBatch _spriteBatch;
    private FreeCamera freeCamera = new FreeCamera(1.0f, Vector3.Forward);

    public Game1()
    {
        _graphics = new GraphicsDeviceManager(this);
        Content.RootDirectory = "Content";
        IsMouseVisible = true;
    }

    private Connection connection;
    protected override void Initialize()
    {
        // TODO: Add your initialization logic here
        connection = new Esp32SerialConnection();

        base.Initialize();
    }

    private BasicEffect _effect;
    protected override void LoadContent()
    {
        _spriteBatch = new SpriteBatch(GraphicsDevice);

        // TODO: use this.Content to load your game content here
        _effect = new BasicEffect(GraphicsDevice)
        {
            VertexColorEnabled = true,
            View = Matrix.CreateLookAt(
                new Vector3(0, 0, 5), Vector3.Zero, Vector3.Up),
            Projection = Matrix.CreatePerspectiveFieldOfView(
                MathHelper.ToRadians(45f),
                GraphicsDevice.Viewport.AspectRatio, 0.1f, 100f)
        };
    }

    List<TrackerData> data = new();
    protected override void Update(GameTime gameTime)
    {
        if (GamePad.GetState(PlayerIndex.One).Buttons.Back == ButtonState.Pressed || Keyboard.GetState().IsKeyDown(Keys.Escape))
        {
            Exit();
        }
        // TODO: Add your update logic here
        data.AddRange(connection.read());
        if(data.Count > short.MaxValue)
        {
            data.RemoveRange(0, data.Count - short.MaxValue);
        }

        KeyboardState kb = Keyboard.GetState();
        MouseState mouse = Mouse.GetState();

        Point screenCentre = new Point(
            GraphicsDevice.Viewport.Width / 2,
            GraphicsDevice.Viewport.Height / 2
        );

        freeCamera.Update(gameTime, kb, mouse, screenCentre);

        base.Update(gameTime);
    }

    protected override void Draw(GameTime gameTime)
    {
        GraphicsDevice.Clear(Color.DarkGray);

        _effect.World = Matrix.Identity;
        _effect.View = freeCamera.View;
        _effect.Projection = freeCamera.Projection;

        if(data.Count >= 2)
        {
            var vertices = new VertexPositionColor[data.Count];
            var vertices_indices = new short[data.Count];
            for(int i = 0; i < data.Count; i++)
            {
                TrackerData point = data[i];
                vertices[i] = new VertexPositionColor(point.get_position(), Color.Red);
                vertices_indices[i] = (short) i;
            }

            foreach (var pass in _effect.CurrentTechnique.Passes)
            {
                pass.Apply();
                GraphicsDevice.DrawUserIndexedPrimitives<VertexPositionColor>(
                    PrimitiveType.LineList,
                    vertices,
                    0,  // vertex buffer offset to add to each element of the index buffer
                    vertices.Length,  // number of vertices in pointList
                    vertices_indices,  // the index buffer
                    0,  // first index element to read
                    vertices.Length >> 1 // number of primitives to draw
                );
            }
        }

        base.Draw(gameTime);
    }
}
