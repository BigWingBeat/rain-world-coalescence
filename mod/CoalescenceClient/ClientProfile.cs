namespace CoalescenceClient
{
	public class ClientProfile
	{
		public string Username { get; private init; }

		public ClientProfile()
		{
			Username = Steamworks.SteamFriends.GetPersonaName();
		}
	}
}
