using Menu;
using Menu.Remix;
using Menu.Remix.MixedUI;
using RWCustom;
using UnityEngine;
using VanillaMenu = Menu.Menu;

namespace MultiplayerMvpClient.Menu
{
	public class MultiplayerLobby : VanillaMenu
	{
		public const string BACK_BUTTON_SIGNAL = "BACK";
		public const string CONNECT_BUTTON_SIGNAL = "CONNECT";
		public const string DISCONNECT_BUTTON_SIGNAL = "DISCONNECT";
		public const string MAIN_MENU_BUTTON_SIGNAL = "MULTIPLAYER";

		public static readonly ProcessManager.ProcessID MultiplayerLobbyId = new("MultiplayerLobby", true);

		private readonly FSprite NativeDot;

		private OpTextBox ServerIpAddress;

		private OpUpdown ServerPort;

		private HoldButton ConnectButton;

		private bool exiting;

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

			ConnectButton = new(this, page, Translate(CONNECT_BUTTON_SIGNAL), CONNECT_BUTTON_SIGNAL, new(screenCenter.x, screenSize.y * 0.3f), 30);
			ConnectButton.GetButtonBehavior.greyedOut = true;
			page.subObjects.Add(ConnectButton);

			// SimpleButton disconnectButton = new(this, page, Translate(DISCONNECT_BUTTON_SIGNAL), DISCONNECT_BUTTON_SIGNAL, new Vector2(500, 280), new Vector2(110, 30));
			// page.subObjects.Add(disconnectButton);

			// IP address and port number
			{
				Vector2 serverSocketAddressAnchor = new(screenCenter.x, screenSize.y * 0.6f);
				MenuTabWrapper tabWrapper = new(this, page);
				page.subObjects.Add(tabWrapper);

				OpLabel serverIpAddressLabel = new(
					serverSocketAddressAnchor,
					new(20, 24),
					Translate("Server Address:"),
					FLabelAlignment.Left);

				float serverIpAddressLabelWidth = serverIpAddressLabel.label.textRect.width;

				const float serverIpAddressWidth = 274;
				ServerIpAddress = new(
					new Configurable<string>(""),
					serverSocketAddressAnchor,
					 serverIpAddressWidth);
				ServerIpAddress.OnValueUpdate += (_, value, _) => ConnectButton.GetButtonBehavior.greyedOut = string.IsNullOrEmpty(value);

				OpLabel serverPortLabel = new(
					serverSocketAddressAnchor,
					 new(20, 24),
					  Translate("Port:"),
					  FLabelAlignment.Left);

				float serverPortLabelWidth = serverPortLabel.label.textRect.width;

				const int upDownOffset = -3;
				const float serverPortWidth = 75;
				ServerPort = new(
					new Configurable<int>(7110, new ConfigAcceptableRange<int>(0, 9999)),
					serverSocketAddressAnchor,
					serverPortWidth);
				ServerPort.PosY += upDownOffset;

				const float padding = 10;
				float totalWidth = serverIpAddressLabelWidth + padding + serverIpAddressWidth + padding + serverPortLabelWidth + padding + serverPortWidth;

				serverIpAddressLabel.PosX = serverSocketAddressAnchor.x - (totalWidth / 2);
				ServerIpAddress.PosX = serverIpAddressLabel.PosX + serverIpAddressLabelWidth + padding;
				serverPortLabel.PosX = ServerIpAddress.PosX + serverIpAddressWidth + padding;
				ServerPort.PosX = serverPortLabel.PosX + serverPortLabelWidth + padding;

				_ = new UIelementWrapper(tabWrapper, serverIpAddressLabel);
				_ = new UIelementWrapper(tabWrapper, ServerIpAddress);
				_ = new UIelementWrapper(tabWrapper, serverPortLabel);
				_ = new UIelementWrapper(tabWrapper, ServerPort);
			}

			// NativeDot = new("pixel")
			// {
			// 	color = Color.white,
			// 	scaleX = 5,
			// 	scaleY = 5,
			// 	x = screenCenter.x,
			// 	y = screenCenter.y
			// };
			// page.Container.AddChild(NativeDot);

			manager.musicPlayer?.FadeOutAllSongs(25f);

			Interop.set_error_handler(DisplayNativeError);
		}

		private static void DisplayNativeError(string text)
		{
			Plugin.Logger.LogInfo($"Native code error: {text}");

			if (Custom.rainWorld?.processManager?.currentMainLoop is MultiplayerLobby menu)
			{
				menu.PlaySound(SoundID.MENU_Security_Button_Release);
				menu.ServerIpAddress.Unassign();
				menu.ServerPort.Unassign();
				DialogNotify dialog = new(text, Custom.rainWorld.processManager, () =>
				{
					menu.PlaySound(SoundID.MENU_Button_Standard_Button_Pressed);
					menu.ServerIpAddress.Assign();
					menu.ServerPort.Assign();
				});
				Custom.rainWorld.processManager.ShowDialog(dialog);
			}
			else
			{
				throw new Exception(text);
			}
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
			bool exitRequested = Interop.update_app();
			if (exitRequested)
			{
				Plugin.Logger.LogInfo("Native app requested exit");
				Interop.destroy_app();
			}
			else
			{
				// MovementDelta delta = MultiplayerMvpNative.query_movement_delta();
				// NativeDot.x += delta.x;
				// NativeDot.y += delta.y;
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
				ExitToMainMenu();
			}
		}

		private void Connect()
		{
			string address = ServerIpAddress.value;
			int port = ServerPort.valueInt;
			// MultiplayerMvpNative.init_app();
			Plugin.Logger.LogInfo($"C# connecting to: {address} on port: {port}");
			Interop.connect_to_server(address, (ushort)port);
		}

		private void Disconnect()
		{
			Interop.destroy_app();
		}

		private void ExitToMainMenu()
		{
			// Exiting out while a Dialog is up softlocks the game, so don't do that
			if (!exiting && manager.dialog == null)
			{
				exiting = true;
				Interop.destroy_app();
				Interop.reset_to_default_error_handler();
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
					Connect();
					break;
				case DISCONNECT_BUTTON_SIGNAL:
					PlaySound(SoundID.MENU_Button_Standard_Button_Pressed);
					Disconnect();
					break;
				case BACK_BUTTON_SIGNAL:
					ExitToMainMenu();
					break;
			}
		}
	}
}
