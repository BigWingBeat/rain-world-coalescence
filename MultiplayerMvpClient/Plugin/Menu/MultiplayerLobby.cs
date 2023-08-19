using Menu;
using Menu.Remix;
using Menu.Remix.MixedUI;
using MultiplayerMvpClient.NativeInterop;
using UnityEngine;
using VanillaMenu = Menu.Menu;

namespace MultiplayerMvpClient.Plugin.Menu
{
	public class MultiplayerLobby : VanillaMenu
	{
		public const string BACK_BUTTON_SIGNAL = "BACK";
		public const string CONNECT_BUTTON_SIGNAL = "CONNECT";
		public const string DISCONNECT_BUTTON_SIGNAL = "DISCONNECT";
		public const string MAIN_MENU_BUTTON_SIGNAL = "MULTIPLAYER";

		public static readonly ProcessManager.ProcessID MultiplayerLobbyId = new("MultiplayerLobby", true);

		private bool exiting;

		private readonly FSprite NativeDot;

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

			SimpleButton backButton = new(this, pages[0], Translate(BACK_BUTTON_SIGNAL), BACK_BUTTON_SIGNAL, new Vector2(200f, 50f), new Vector2(110f, 30f));
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

			SimpleButton connectButton = new(this, pages[0], Translate(CONNECT_BUTTON_SIGNAL), CONNECT_BUTTON_SIGNAL, new Vector2(500, 380), new Vector2(110, 30));
			pages[0].subObjects.Add(connectButton);

			SimpleButton disconnectButton = new(this, pages[0], Translate(DISCONNECT_BUTTON_SIGNAL), DISCONNECT_BUTTON_SIGNAL, new Vector2(500, 280), new Vector2(110, 30));
			pages[0].subObjects.Add(disconnectButton);

			NativeDot = new("pixel")
			{
				color = Color.white,
				scaleX = 5,
				scaleY = 5,
				x = screenCenter.x,
				y = screenCenter.y
			};
			pages[0].Container.AddChild(NativeDot);

			manager.musicPlayer?.FadeOutAllSongs(25f);
		}

		public static void SetupHooks()
		{
			On.Menu.MainMenu.ctor += AddMainMenuButton;
			On.ProcessManager.PostSwitchMainProcess += SwitchMainProcess;
		}

		private static void AddMainMenuButton(On.Menu.MainMenu.orig_ctor orig, MainMenu self, ProcessManager manager, bool showRegionSpecificBkg)
		{
			orig(self, manager, showRegionSpecificBkg);

			float buttonWidth = MainMenu.GetButtonWidth(self.CurrLang);
			Vector2 pos = new(683f - (buttonWidth / 2f), 0f);
			Vector2 size = new(buttonWidth, 30f);
			int indexFromBottomOfList = 5;
			self.AddMainMenuButton(new SimpleButton(self, self.pages[0], self.Translate(MAIN_MENU_BUTTON_SIGNAL), MAIN_MENU_BUTTON_SIGNAL, pos, size), () => MainMenuButtonPressed(self), indexFromBottomOfList);
		}

		private static void MainMenuButtonPressed(VanillaMenu from)
		{
			from.manager.RequestMainProcessSwitch(MultiplayerLobbyId);
			from.PlaySound(SoundID.MENU_Switch_Page_In);
		}

		private static void SwitchMainProcess(On.ProcessManager.orig_PostSwitchMainProcess orig, ProcessManager self, ProcessManager.ProcessID ID)
		{
			if (ID == MultiplayerLobbyId)
			{
				self.currentMainLoop = new MultiplayerLobby(self, ID);
			}
			orig(self, ID);
		}

		public override void RawUpdate(float dt)
		{
			bool exitRequested = MultiplayerMvpNative.update_app();
			if (exitRequested)
			{
				MultiplayerMvpClientPlugin.Logger.LogInfo("Native app requested exit");
				MultiplayerMvpNative.destroy_app();
			}
			else
			{
				MovementDelta delta = MultiplayerMvpNative.query_movement_delta();
				MultiplayerMvpClientPlugin.Logger.LogInfo($"Queried delta of: {delta}");
				NativeDot.x += delta.x;
				NativeDot.y += delta.y;
			}

			base.RawUpdate(dt);
		}

		public override void Update()
		{
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

		private void OnExit()
		{
			if (!exiting)
			{
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
				case CONNECT_BUTTON_SIGNAL:
					MultiplayerMvpClientPlugin.Logger.LogInfo($"Connecting");
					MultiplayerMvpNative.init_app();
					break;
				case DISCONNECT_BUTTON_SIGNAL:
					MultiplayerMvpClientPlugin.Logger.LogInfo($"Disconnecting");
					MultiplayerMvpNative.destroy_app();
					break;
				case BACK_BUTTON_SIGNAL:
					MultiplayerMvpClientPlugin.Logger.LogInfo($"Backing out");
					OnExit();
					break;
			}
		}
	}
}
