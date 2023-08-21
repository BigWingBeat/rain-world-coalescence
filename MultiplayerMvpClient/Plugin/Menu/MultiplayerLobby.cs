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

		private bool hasDoneInitUpdate;

		public override bool FreezeMenuFunctions => hasDoneInitUpdate && base.FreezeMenuFunctions;

		public MultiplayerLobby(ProcessManager manager, ProcessManager.ProcessID ID) : base(manager, ID)
		{
			Vector2 screenSize = manager.rainWorld.screenSize;
			Vector2 screenCenter = manager.rainWorld.screenSize / 2;

			Vector2 bottomLeft = Vector2.zero;
			Vector2 topRight = screenSize;

			Vector2 standardButtonSize = new(110f, 30f);

			Page page = new(this, null, "main", 0);
			pages.Add(page);

			scene = new InteractiveMenuScene(this, page, ModManager.MMF ? manager.rainWorld.options.subBackground : MenuScene.SceneID.Landscape_SU);
			page.subObjects.Add(scene);

			FSprite backgroundTint = new("pixel")
			{
				color = Color.black,
				anchorX = 0,
				anchorY = 0,
				scaleX = screenSize.x + 2,
				scaleY = screenSize.y + 2,
				x = -1,
				y = -1,
				alpha = 0.85f
			};
			page.Container.AddChild(backgroundTint);

			MenuIllustration title = new(this, scene, "", "MultiplayerTitle", Vector2.zero, crispPixels: true, anchorCenter: false);
			title.sprite.shader = manager.rainWorld.Shaders["MenuText"];
			scene.AddIllustration(title);

			SimpleButton backButton = new(this, page, Translate(BACK_BUTTON_SIGNAL), BACK_BUTTON_SIGNAL, new Vector2(200f, 50f), standardButtonSize);
			page.subObjects.Add(backButton);
			backObject = backButton;

			// IP address and port number
			{
				Vector2 serverSocketAddressAnchor = new(screenCenter.x, screenSize.y * 0.6f);
				MenuTabWrapper tabWrapper = new(this, page);
				page.subObjects.Add(tabWrapper);

				OpLabel serverAddressLabel = new(
					serverSocketAddressAnchor,
					new(20, 24),
					"Server Address:",
					FLabelAlignment.Left);

				float serverAddressLabelWidth = serverAddressLabel.label.textRect.width;

				const float serverAddressWidth = 274;
				OpTextBox serverAddress = new(
					new Configurable<string>(""),
					serverSocketAddressAnchor,
					 serverAddressWidth);

				OpLabel serverPortLabel = new(
					serverSocketAddressAnchor,
					 new(20, 24),
					  "Port:",
					  FLabelAlignment.Left);

				float serverPortLabelWidth = serverPortLabel.label.textRect.width;

				const int upDownOffset = -3;
				const float serverPortWidth = 75;
				OpUpdown serverPort = new(
					new Configurable<int>(7110, new ConfigAcceptableRange<int>(0, 9999)),
					serverSocketAddressAnchor,
					serverPortWidth);
				serverPort.PosY += upDownOffset;

				const float padding = 10;
				float totalWidth = serverAddressLabelWidth + padding + serverAddressWidth + padding + serverPortLabelWidth + padding + serverPortWidth;

				serverAddressLabel.PosX = serverSocketAddressAnchor.x - (totalWidth / 2);
				serverAddress.PosX = serverAddressLabel.PosX + serverAddressLabelWidth + padding;
				serverPortLabel.PosX = serverAddress.PosX + serverAddressWidth + padding;
				serverPort.PosX = serverPortLabel.PosX + serverPortLabelWidth + padding;

				_ = new UIelementWrapper(tabWrapper, serverAddressLabel);
				_ = new UIelementWrapper(tabWrapper, serverAddress);
				_ = new UIelementWrapper(tabWrapper, serverPortLabel);
				_ = new UIelementWrapper(tabWrapper, serverPort);
			}

			SimpleButton connectButton = new(this, page, Translate(CONNECT_BUTTON_SIGNAL), CONNECT_BUTTON_SIGNAL, new Vector2(500, 380), new Vector2(110, 30));
			page.subObjects.Add(connectButton);

			SimpleButton disconnectButton = new(this, page, Translate(DISCONNECT_BUTTON_SIGNAL), DISCONNECT_BUTTON_SIGNAL, new Vector2(500, 280), new Vector2(110, 30));
			page.subObjects.Add(disconnectButton);

			NativeDot = new("pixel")
			{
				color = Color.white,
				scaleX = 5,
				scaleY = 5,
				x = screenCenter.x,
				y = screenCenter.y
			};
			page.Container.AddChild(NativeDot);

			manager.musicPlayer?.FadeOutAllSongs(25f);
		}

		internal static void SetupHooks()
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

		public override void Init()
		{
			base.Init();
			init = true;
			Update();
			hasDoneInitUpdate = true;
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
