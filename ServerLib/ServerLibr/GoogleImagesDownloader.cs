using System;
using System.Net;
using System.Text.RegularExpressions;
using System.Threading.Tasks;

namespace ServerLib
{
	public class GoogleImagesDownloader
	{
		public GoogleImagesDownloader ()
		{

		}


		async public Task<string> GetImageFor(string searchstring)
		{
			try {
				// https://www.google.de/search?tbm=isch&q=toast&tbs=imgo:1&gws_rd=cr&dcr=0&ei=tppsWonOFs6YkwXduZLoBA
				// <img height="100" src="https://encrypted-tbn0.gstatic.com/images?q=tbn:ANd9GcTTnvw-NnkfX0WoR3JAXOpsaDC6zXgwlvZ-xlNqkaq5fXVzAnsunO0iq2ND" width="150" alt="Bildergebnis f�r Toast">
				
				using(WebClient client = new WebClient()) {
					string source = await client.DownloadStringTaskAsync("https://www.google.de/search?tbm=isch&q=" + WebUtility.UrlEncode(searchstring) + "&tbs=imgo:1&gws_rd=cr&dcr=0");
					source = WebUtility.HtmlDecode(source);
					Regex img_finder = new Regex("<img height=\"[0-9]+\" src=\"(https://encrypted[^\"]*)\" [^>]*>");
					string matches = "";
					foreach (Match match in img_finder.Matches(source)) {
						//matches += match.Value + "\n";
						matches += match.Groups [1] + "\n";
					}
					
					return matches;
				}
				
			} catch (Exception ex) {
				return ex.ToString ();
			}
		}
	}
}

