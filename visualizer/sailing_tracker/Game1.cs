using System;
using System.Collections.Generic;
using System.Net.Http;
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
    private FreeCamera freeCamera = new FreeCamera(TargetAspectRatio, Vector3.Backward);

    VertexPositionColor[] axes = new[]
    {
        new VertexPositionColor(Vector3.Zero, Color.Red),
        new VertexPositionColor(Vector3.UnitX * 2f, Color.Red),

        new VertexPositionColor(Vector3.Zero, Color.Green),
        new VertexPositionColor(Vector3.UnitY * 2f, Color.Green),

        new VertexPositionColor(Vector3.Zero, Color.Blue),
        new VertexPositionColor(Vector3.UnitZ * 2f, Color.Blue)
    };

    private const float TargetAspectRatio = 1f / 1f;
    private void SetAspectRatioViewport()
    {
        int width = Window.ClientBounds.Width;
        int height = Window.ClientBounds.Height;

        if (width <= 0 || height <= 0)
            return;

        float windowAspectRatio = (float)width / height;
        int viewportWidth;
        int viewportHeight;

        if (windowAspectRatio > TargetAspectRatio)
        {
            viewportHeight = height;
            viewportWidth = (int)(height * TargetAspectRatio);
        }
        else
        {
            viewportWidth = width;
            viewportHeight = (int)(width / TargetAspectRatio);
        }

        GraphicsDevice.Viewport = new Viewport(
            (width - viewportWidth) / 2,
            (height - viewportHeight) / 2,
            viewportWidth,
            viewportHeight
        );
    }

    public Game1()
    {
        _graphics = new GraphicsDeviceManager(this);

        _graphics.PreferredBackBufferWidth = 1280;
        _graphics.PreferredBackBufferHeight = 720;
        _graphics.IsFullScreen = false;

        Window.AllowUserResizing = true;

        Content.RootDirectory = "Content";
        IsMouseVisible = false;
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
        };
    }

    private List<TrackerData> data = new();
    private bool focused = true;
    private KeyboardState previous_kb_state = Keyboard.GetState();
    protected override void Update(GameTime gameTime)
    {
        KeyboardState kb = Keyboard.GetState();
        MouseState mouse = Mouse.GetState();

        if (GamePad.GetState(PlayerIndex.One).Buttons.Back == ButtonState.Pressed || (kb.IsKeyDown(Keys.LeftAlt) && kb.IsKeyDown(Keys.F4)))
        {
            Exit();
        }

        Point screenCentre = new Point(
            GraphicsDevice.Viewport.Width / 2,
            GraphicsDevice.Viewport.Height / 2
        );
        
        if(kb.IsKeyDown(Keys.Escape) && previous_kb_state.IsKeyUp(Keys.Escape))
        {
            focused = !focused;
            if(focused)
            {
                IsMouseVisible = false;
            }
            else
            {
                IsMouseVisible = true;
                Mouse.SetPosition(screenCentre.X, screenCentre.Y);
            }
        }

        data.AddRange(connection.read());
        if(data.Count > short.MaxValue)
        {
            data.RemoveRange(0, data.Count - short.MaxValue);
        }



        if(focused)
        {
            freeCamera.Update(gameTime, kb, mouse, screenCentre);
            Mouse.SetPosition(screenCentre.X, screenCentre.Y);
        }

        previous_kb_state = kb;

        base.Update(gameTime);
    }

    protected override void Draw(GameTime gameTime)
    {
        if(focused)
        {
            GraphicsDevice.Clear(new Color(90, 90, 90));
        } 
        else
        {
            GraphicsDevice.Clear(new Color(10, 10, 10));
        }

        SetAspectRatioViewport();

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

        foreach (var pass in _effect.CurrentTechnique.Passes)
        {
            pass.Apply();
            GraphicsDevice.DrawUserPrimitives(
                PrimitiveType.LineList,
                axes,
                0,
                3 // three line segments
            );
        }

        base.Draw(gameTime);
    }
}
