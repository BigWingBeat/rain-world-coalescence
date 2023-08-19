using Menu;
using Menu.Remix;
using Menu.Remix.MixedUI;
using MultiplayerMvpClient.NativeInterop;
using UnityEngine;

namespace MultiplayerMvpClient.Plugin.Menu
{
    public class MultiplayerLobby : global::Menu.Menu
    {
        public static ProcessManager.ProcessID MultiplayerLobbyId = new("MultiplayerLobby", true);

        private bool exiting = false;

        private FSprite nativeDot;

        public const string BACK_BUTTON_MESSAGE = "BACK";

        public const string CONNECT_BUTTON_MESSAGE = "CONNECT";

        public const string DISCONNECT_BUTTON_MESSAGE = "DISCONNECT";

        public MultiplayerLobby(ProcessManager manager, ProcessManager.ProcessID ID) : base(manager, ID)
        {
            pages.Add(new Page(this, null, "main", 0));
            scene = new InteractiveMenuScene(this, pages[0], ModManager.MMF ? manager.rainWorld.options.subBackground : MenuScene.SceneID.Landscape_SU);
            pages[0].subObjects.Add(scene);

            FSprite backgroundTint = new("pixel")
            {
                color = Color.black,
                anchorX = 0f,
                anchorY = 0f,
                scaleX = 1368f,
                scaleY = 770f,
                x = -1f,
                y = -1f,
                alpha = 0.85f
            };
            pages[0].Container.AddChild(backgroundTint);

            scene.AddIllustration(new MenuIllustration(this, scene, "", "MultiplayerTitle", new Vector2(-2.99f, 265.01f), crispPixels: true, anchorCenter: false));
            scene.flatIllustrations.Last().sprite.shader = manager.rainWorld.Shaders["MenuText"];
            scene.flatIllustrations.Last().pos.x += HorizontalMoveToGetCentered(manager);

            SimpleButton backButton = new(this, pages[0], Translate(BACK_BUTTON_MESSAGE), BACK_BUTTON_MESSAGE, new Vector2(200f, 50f), new Vector2(110f, 30f));
            pages[0].subObjects.Add(backButton);
            backObject = backButton;

            Vector2 screenCenter = manager.rainWorld.screenSize / 2;

            MenuTabWrapper tabWrapper = new(this, pages[0]);
            pages[0].subObjects.Add(tabWrapper);

            OpTextBox textbox = new(new Configurable<string>(""), new(500, 500), 200)
            {
                allowSpace = true
            };
            _ = new UIelementWrapper(tabWrapper, textbox);

            SimpleButton connectButton = new(this, pages[0], Translate(CONNECT_BUTTON_MESSAGE), CONNECT_BUTTON_MESSAGE, new Vector2(500, 380), new Vector2(110, 30));
            pages[0].subObjects.Add(connectButton);

            SimpleButton disconnectButton = new(this, pages[0], Translate(DISCONNECT_BUTTON_MESSAGE), DISCONNECT_BUTTON_MESSAGE, new Vector2(500, 280), new Vector2(110, 30));
            pages[0].subObjects.Add(disconnectButton);

            nativeDot = new("pixel")
            {
                color = Color.white,
                scaleX = 5,
                scaleY = 5,
                x = screenCenter.x,
                y = screenCenter.y
            };
            pages[0].Container.AddChild(nativeDot);

            manager.musicPlayer?.FadeOutAllSongs(25f);
        }

        public static void SetupHooks()
        {
            On.Menu.MainMenu.ctor += AddMainMenuButton;
            On.ProcessManager.PostSwitchMainProcess += SwitchMainProcess;
        }

        public static void AddMainMenuButton(On.Menu.MainMenu.orig_ctor orig, MainMenu self, ProcessManager manager, bool showRegionSpecificBkg)
        {
            orig(self, manager, showRegionSpecificBkg);

            float buttonWidth = MainMenu.GetButtonWidth(self.CurrLang);
            Vector2 pos = new(683f - (buttonWidth / 2f), 0f);
            Vector2 size = new(buttonWidth, 30f);
            self.AddMainMenuButton(new SimpleButton(self, self.pages[0], self.Translate("MULTIPLAYER"), "MULTIPLAYER", pos, size), () => MultiplayerButtonPressed(self), 5);
        }

        public static void SwitchMainProcess(On.ProcessManager.orig_PostSwitchMainProcess orig, ProcessManager self, ProcessManager.ProcessID ID)
        {
            if (ID == MultiplayerLobbyId)
            {
                self.currentMainLoop = new MultiplayerLobby(self, ID);
            }
            orig(self, ID);
        }

        private static void MultiplayerButtonPressed(global::Menu.Menu from)
        {
            from.manager.RequestMainProcessSwitch(MultiplayerLobbyId);
            from.PlaySound(SoundID.MENU_Switch_Page_In);
        }

        public override void RawUpdate(float dt)
        {
            bool exitRequested = MultiplayerMvpNative.update_app();
            MultiplayerMvpClientPlugin.logger.LogInfo($"RawUpdate (exitRequested: {exitRequested})");
            if (exitRequested)
            {
                MultiplayerMvpNative.destroy_app();
            }

            base.RawUpdate(dt);
        }

        public override void Update()
        {
            MultiplayerMvpClientPlugin.logger.LogInfo($"Update");
            base.Update();

            if (manager.musicPlayer?.song == null)
            {
                manager.musicPlayer?.MenuRequestsSong("RW_43 - Bio Engineering", 1f, 1f);
            }

            if (RWInput.CheckPauseButton(0, manager.rainWorld))
            {
                OnExit();
            }
        }

        public void DoMovement(float x_diff, float y_diff)
        {
            nativeDot.x += x_diff;
            nativeDot.y += y_diff;
            MultiplayerMvpClientPlugin.logger.LogInfo($"C# callback with {x_diff}, {y_diff}");
        }

        public void OnExit()
        {
            MultiplayerMvpClientPlugin.logger.LogInfo($"Exiting");
            if (!exiting)
            {
                MultiplayerMvpClientPlugin.logger.LogInfo($"Actually exiting");
                exiting = true;
                MultiplayerMvpNative.destroy_app();
                PlaySound(SoundID.MENU_Switch_Page_Out);
                manager.musicPlayer?.FadeOutAllSongs(100f);
                manager.RequestMainProcessSwitch(ProcessManager.ProcessID.MainMenu);
            }
        }

        public override void Singal(MenuObject sender, string message)
        {
            switch (message)
            {
                case CONNECT_BUTTON_MESSAGE:
                    MultiplayerMvpClientPlugin.logger.LogInfo($"Connecting");
                    MultiplayerMvpNative.init_app(DoMovement);
                    break;
                case DISCONNECT_BUTTON_MESSAGE:
                    MultiplayerMvpClientPlugin.logger.LogInfo($"Disconnecting");
                    MultiplayerMvpNative.destroy_app();
                    break;
                case BACK_BUTTON_MESSAGE:
                    MultiplayerMvpClientPlugin.logger.LogInfo($"Backing out");
                    OnExit();
                    break;
            }
        }
    }
}