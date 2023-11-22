using System.Runtime.InteropServices;
using Menu;
using Menu.Remix.MixedUI;
using UnityEngine;

namespace MultiplayerMvpClient.Menu
{
	// The menu used to find and connect to servers. Accessed from the main menu.
	// Can directly connect to a specified IP/domain + port, and in future will have a steam-integrated server browser
	public unsafe class ServerBrowserMenu : CustomMenuBase
	{
		public const string CONNECT_BUTTON_SIGNAL = "CONNECT";
		public const string DISCONNECT_BUTTON_SIGNAL = "DISCONNECT";
		public const string MAIN_MENU_BUTTON_SIGNAL = "MULTIPLAYER";

		public static readonly ProcessManager.ProcessID ProcessId = new(nameof(ServerBrowserMenu), true);

		private OpTextBox ServerIpAddress;

		private OpUpdown ServerPort;

		private HoldButton ConnectButton;

		private AppContainer* appHandle = null;
		private ConnectionTask* connectionTaskHandle = null;

		public ServerBrowserMenu(ProcessManager manager, ProcessManager.ProcessID ID) : base(manager, ID)
		{
			var builder = new CustomMenuBuilder()
				.WithBackgroundArt(true)
				.WithTitleIllustration("MultiplayerTitle")
				.WithStandardBackButton()
				.WithSong("RW_43 - Bio Engineering");

			// Connect to server button
			// Starts greyed out as the IP text box starts empty
			ConnectButton = new(this, null, Translate(CONNECT_BUTTON_SIGNAL), CONNECT_BUTTON_SIGNAL, new(ScreenDimensions.ScreenCenter.x, ScreenDimensions.ScreenCenter.y * 0.3f), 30);
			ConnectButton.GetButtonBehavior.greyedOut = true;
			builder.WithMenuObject(ConnectButton);

			// IP address and port number, layed out to be centered
			{
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

				builder.WithMixedUiElement(serverIpAddressLabel)
					.WithMixedUiElement(ServerIpAddress)
					.WithMixedUiElement(serverPortLabel)
					.WithMixedUiElement(ServerPort);
			}

			builder.Build(this);
		}

		internal static void SetupHooks()
		{
			SetupMainMenuButtonHook(5, ProcessId, MAIN_MENU_BUTTON_SIGNAL);
			SetupSwitchMainProcessHook(ProcessId, (manager, ID) => new ServerBrowserMenu(manager, ID));
		}

		public override void RawUpdate(float dt)
		{
			if (appHandle != null)
			{
				bool exitRequested = Convert.ToBoolean(Interop.update_app(appHandle));
				if (exitRequested)
				{
					Plugin.Logger.LogInfo("Native app requested exit");
					Interop.drop_app(appHandle);
					appHandle = null;
					if (connectionTaskHandle != null)
					{
						Interop.drop_connection_task(connectionTaskHandle);
						connectionTaskHandle = null;
					}
				}
			}

			if (connectionTaskHandle != null)
			{
				PollConnectionTaskResult pollResult;
				pollResult = Interop.poll_connection_task(connectionTaskHandle);
				switch (pollResult.tag)
				{
					case PollConnectionTaskResult.Tag.Ok:
						Plugin.Logger.LogInfo("Server connection completed successfully");
						Interop.drop_connection_task(connectionTaskHandle);
						connectionTaskHandle = null;
						break;
					case PollConnectionTaskResult.Tag.Err:
						Interop.drop_connection_task(connectionTaskHandle);
						connectionTaskHandle = null;
						DisplayNativeError(InteropUtils.FormatNativeError(pollResult.err._0));
						break;
				}
			}

			base.RawUpdate(dt);
		}

		private void Connect()
		{
			string address = ServerIpAddress.value;
			ushort port = (ushort)ServerPort.valueInt;
			Plugin.Logger.LogInfo($"Connecting to: {address} on port: {port}");

			if (appHandle != null)
			{
				Interop.drop_app(appHandle);
			}
			appHandle = Interop.new_app();

			IntPtr addressPointer = Marshal.StringToHGlobalUni(address);
			AppConnectToServerResult result = Interop.app_connect_to_server(appHandle, (ushort*)addressPointer, port);
			Marshal.FreeHGlobal(addressPointer);

			switch (result.tag)
			{
				case AppConnectToServerResult.Tag.Ok:
					connectionTaskHandle = result.ok._0;
					break;
				case AppConnectToServerResult.Tag.Err:
					DisplayNativeError(InteropUtils.FormatNativeError(result.err._0));
					break;
			}
		}

		private void Disconnect()
		{
			if (appHandle != null)
			{
				Interop.drop_app(appHandle);
				appHandle = null;
			}

			if (connectionTaskHandle != null)
			{
				Interop.drop_connection_task(connectionTaskHandle);
				connectionTaskHandle = null;
			}
		}

		private void ExitToMainMenu()
		{
			if (!IsSwitchingMainProcess)
			{
				Disconnect();
				BackOutMainProcessSwitch(ProcessManager.ProcessID.MainMenu);
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
