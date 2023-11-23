using Menu;

namespace MultiplayerMvpClient.Menu
{
	public abstract class SinglePageMenu : CustomMenuBase
	{
		public virtual Page Page { get; set; }

		protected SinglePageMenu(ProcessManager manager, ProcessManager.ProcessID ID, Page? page = null) : base(manager, ID)
		{
			Page = page ?? new(this, null, "main", 0);
			pages.Add(Page);
			pages.TrimExcess();
		}
	}
}
