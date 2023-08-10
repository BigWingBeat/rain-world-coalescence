using Menu;
using UnityEngine;

namespace multiplayer_mvp.plugin.menu
{
    public class MultiplayerLobby : Menu.Menu
    {
        public static ProcessManager.ProcessID MultiplayerLobbyId = new("MultiplayerLobby", true);

        private bool exiting = false;

        public MultiplayerLobby(ProcessManager manager, ProcessManager.ProcessID ID) : base(manager, ID)
        {
            pages.Add(new Page(this, null, "main", 0));
            scene = new InteractiveMenuScene(this, pages[0], ModManager.MMF ? manager.rainWorld.options.subBackground : MenuScene.SceneID.Landscape_SU);
            pages[0].subObjects.Add(scene);

            FSprite backgroundTint = new("pixel")
            {
                color = new Color(0f, 0f, 0f),
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

            SimpleButton backButton = new(this, pages[0], Translate("BACK"), "BACK", new Vector2(200f, 50f), new Vector2(110f, 30f));
            pages[0].subObjects.Add(backButton);
            backObject = backButton;

            manager.musicPlayer?.FadeOutAllSongs(25f);
        }

        public static void AddMainMenuButton(On.Menu.MainMenu.orig_ctor orig, Menu.MainMenu self, ProcessManager manager, bool showRegionSpecificBkg)
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

        private static void MultiplayerButtonPressed(Menu.Menu from)
        {
            from.manager.RequestMainProcessSwitch(MultiplayerLobbyId);
            from.PlaySound(SoundID.MENU_Switch_Page_In);
        }

        public override void Update()
        {
            base.Update();

            manager.musicPlayer?.MenuRequestsSong("RW_43 - Bio Engineering", 1f, 1f);

            if (RWInput.CheckPauseButton(0, manager.rainWorld))
            {
                OnExit();
            }
        }

        public void OnExit()
        {
            if (!exiting)
            {
                exiting = true;
                PlaySound(SoundID.MENU_Switch_Page_Out);
                manager.musicPlayer?.FadeOutAllSongs(100f);
                manager.RequestMainProcessSwitch(ProcessManager.ProcessID.MainMenu);
            }
        }

        public override void Singal(MenuObject sender, string message)
        {
            switch (message)
            {
                case "BACK":
                    OnExit();
                    break;
            }
        }
    }
}