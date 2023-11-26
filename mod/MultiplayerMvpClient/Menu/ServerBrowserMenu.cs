using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using Menu;
using Menu.Remix.MixedUI;
using UnityEngine;

namespace MultiplayerMvpClient.Menu
{
	// The menu used to find and connect to servers. Accessed from the main menu.
	// Can directly connect to a specified IP/domain + port, and in future will have a steam-integrated server browser
	public unsafe class ServerBrowserMenu : SinglePageMenu
	{
		public const string CONNECT_BUTTON_SIGNAL = "CONNECT";
		public const string MAIN_MENU_BUTTON_SIGNAL = "MULTIPLAYER";

		public static readonly ProcessManager.ProcessID ProcessId = new(nameof(ServerBrowserMenu), true);

#pragma warning disable CS8618 // Gets assigned to in instance constructor
		private static ServerBrowserMenu Instance;
#pragma warning restore

		private OpTextBox ServerIpAddress;

		private OpUpdown ServerPort;

		private HoldButton ConnectButton;

		private SafeAppHandle? appHandle;

		private ClientProfile Profile;

		private bool WaitingForConnection;

		public override bool FreezeMenuFunctions => WaitingForConnection || base.FreezeMenuFunctions;

#pragma warning disable CS8618 // The fields get assigned to in the yield methods, which are called by the builder
		public ServerBrowserMenu(ProcessManager manager, ProcessManager.ProcessID ID) : base(manager, ID)
		{
			Instance = this;
			Profile = new();
			new CustomMenuBuilder()
				.WithBackgroundArt(true)
				.WithTitleIllustration("MultiplayerTitle")
				.WithStandardBackButton()
				.WithSong("RW_43 - Bio Engineering")
				.Build(this, Page);
		}
#pragma warning restore CS8618

		public override IEnumerable<MenuObject> YieldMenuObjects()
		{
			// Connect to server button
			// Starts greyed out as the IP text box starts empty
			ConnectButton = new(this, Page, Translate(CONNECT_BUTTON_SIGNAL), CONNECT_BUTTON_SIGNAL, new(ScreenDimensions.ScreenCenter.x, ScreenDimensions.ScreenSize.y * 0.3f), 30);
			ConnectButton.GetButtonBehavior.greyedOut = true;
			yield return ConnectButton;
		}

		public override IEnumerable<UIelement> YieldMixedUiElements()
		{
			// IP address and port number, layed out to be centered
			Vector2 serverSocketAddressAnchor = new(ScreenDimensions.ScreenCenter.x, ScreenDimensions.ScreenSize.y * 0.6f);

			OpLabel serverIpAddressLabel = new(
				serverSocketAddressAnchor,
				new(20, 24),
				Translate("Server Address:"),
				FLabelAlignment.Left);

			float serverIpAddressLabelWidth = serverIpAddressLabel.label.textRect.width;

			// Just large enough to fit a full IPv6 address
			const float serverIpAddressWidth = 274;
			ServerIpAddress = new(
				new Configurable<string>(""),
				serverSocketAddressAnchor,
				 serverIpAddressWidth);

			// Disable connect button when the IP address textbox is empty
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
				new Configurable<int>(Interop.DEFAULT_PORT, new ConfigAcceptableRange<int>(0, 9999)),
				serverSocketAddressAnchor,
				serverPortWidth);
			ServerPort.PosY += upDownOffset;

			const float padding = 10;
			float totalWidth = serverIpAddressLabelWidth + padding + serverIpAddressWidth + padding + serverPortLabelWidth + padding + serverPortWidth;

			// Lay out the elements relative to eachother so they're centered
			serverIpAddressLabel.PosX = serverSocketAddressAnchor.x - (totalWidth / 2);
			ServerIpAddress.PosX = serverIpAddressLabel.PosX + serverIpAddressLabelWidth + padding;
			serverPortLabel.PosX = ServerIpAddress.PosX + serverIpAddressWidth + padding;
			ServerPort.PosX = serverPortLabel.PosX + serverPortLabelWidth + padding;

			yield return serverIpAddressLabel;
			yield return ServerIpAddress;
			yield return serverPortLabel;
			yield return ServerPort;
		}

		internal static void SetupHooks()
		{
			SetupMainMenuButtonHook(5, ProcessId, MAIN_MENU_BUTTON_SIGNAL);
			SetupSwitchMainProcessHook(ProcessId, (manager, ID) => new ServerBrowserMenu(manager, ID));
		}

		public override void Update()
		{
			base.Update();
			if (WaitingForConnection)
			{
				infoLabelFade = 1;
			}
		}

		public override void RawUpdate(float dt)
		{
			if (appHandle != null && !appHandle.IsInvalid && !appHandle.IsClosed)
			{
				bool exitRequested = appHandle.Update();
				if (exitRequested)
				{
					Plugin.Logger.LogInfo("Native app requested exit");
					appHandle.Close();
					appHandle = null;
				}
			}

			base.RawUpdate(dt);
		}

		public override string UpdateInfoText()
		{
			if (WaitingForConnection)
			{
				return "Connecting...";
			}
			else
			{
				return base.UpdateInfoText();
			}
		}

		public override void CommunicateWithUpcomingProcess(MainLoopProcess nextProcess)
		{
			base.CommunicateWithUpcomingProcess(nextProcess);
			if (nextProcess is ServerLobbyMenu lobby)
			{
				lobby.CommunicateWithPreviousProcess(appHandle, Profile);
			}
		}

		private void Connect()
		{
			string address = ServerIpAddress.value;
			ushort port = (ushort)ServerPort.valueInt;
			Plugin.Logger.LogInfo($"Connecting to: {address} on port: {port}");

			if (appHandle == null || appHandle.IsInvalid || appHandle.IsClosed)
			{
				appHandle = new(Interop.new_app());
			}

			AppConnectToServerResult result = appHandle.ConnectToServer(address, port, Profile.Username, &ConnectedToServerCallback, &NativeErrorCallback);

			switch (result.tag)
			{
				case AppConnectToServerResult.Tag.Ok:
					WaitingForConnection = true;
					DisableTypeables();
					infoLabel.text = UpdateInfoText();
					break;
				case AppConnectToServerResult.Tag.AppPointerIsNull:
					DisplayNativeError("appHandle is null");
					break;
				case AppConnectToServerResult.Tag.AddressPointerIsNull:
					DisplayNativeError("addressPointer is null");
					break;
				case AppConnectToServerResult.Tag.Err:
					DisplayNativeError(InteropUtils.FormatNativeError(result.err._0));
					break;
			}
		}

		[UnmanagedCallersOnly(CallConvs = [typeof(CallConvCdecl)])]
		private static void ConnectedToServerCallback()
		{
			Instance.PlaySound(SoundID.MENU_Start_New_Game);
			Instance.manager.musicPlayer?.FadeOutAllSongs(100f);
			Instance.SwitchMainProcess(ServerLobbyMenu.ProcessId);
		}

		[UnmanagedCallersOnly(CallConvs = [typeof(CallConvCdecl)])]
		private static void NativeErrorCallback(Error* error)
		{
			Instance.WaitingForConnection = false;
			Instance.infoLabel.text = Instance.UpdateInfoText();
			Instance.DisplayNativeError(InteropUtils.FormatNativeError(error));
		}

		private void ExitToMainMenu()
		{
			if (!IsSwitchingMainProcess && !WaitingForConnection)
			{
				appHandle?.Close();
				appHandle = null;
				PlaySound(SoundID.MENU_Switch_Page_Out);
				manager.musicPlayer?.FadeOutAllSongs(100f);
				SwitchMainProcess(ProcessManager.ProcessID.MainMenu);
			}
		}

		public override void Singal(MenuObject sender, string message)
		{
			switch (message)
			{
				case CONNECT_BUTTON_SIGNAL:
					Connect();
					break;
				case BACK_BUTTON_SIGNAL:
					ExitToMainMenu();
					break;
			}
		}
	}
}
