#set document(title: "Scaleway Nix Binary Cache Infrastructure Requirements Research")
#set heading(numbering: "1.")

#let refs = bytes(
  ```bib
  @online{opentofu-faq,
    title = {FAQ},
    author = {{OpenTofu}},
    year = {2026},
    url = {https://opentofu.org/faq/},
    urldate = {2026-04-01}
  }

  @online{opentofu-sensitive-state,
    title = {Sensitive Data in State},
    author = {{OpenTofu}},
    year = {2026},
    url = {https://opentofu.org/docs/language/state/sensitive-data/},
    urldate = {2026-04-01}
  }

  @online{opentofu-s3-backend,
    title = {Backend Type: s3},
    author = {{OpenTofu}},
    year = {2026},
    url = {https://opentofu.org/docs/v1.9/language/settings/backends/s3/},
    urldate = {2026-04-01}
  }

  @online{nix-s3-store,
    title = {S3 Binary Cache Store},
    author = {{nix.dev contributors}},
    year = {2026},
    url = {https://nix.dev/manual/nix/latest/store/types/s3-binary-cache-store},
    urldate = {2026-04-01}
  }

  @online{nix-conf,
    title = {nix.conf - Nix Reference Manual},
    author = {{nix.dev contributors}},
    year = {2026},
    url = {https://nix.dev/manual/nix/latest/command-ref/conf-file.html},
    urldate = {2026-04-01}
  }

  @online{nix-generate-key,
    title = {nix-store --generate-binary-cache-key - Nix Reference Manual},
    author = {{nix.dev contributors}},
    year = {2026},
    url = {https://nix.dev/manual/nix/2.34/command-ref/nix-store/generate-binary-cache-key},
    urldate = {2026-04-01}
  }

  @online{scw-iam-application,
    title = {Resource: scaleway_iam_application},
    author = {{Scaleway}},
    year = {2026},
    url = {https://github.com/scaleway/terraform-provider-scaleway/blob/main/docs/resources/iam_application.md},
    urldate = {2026-04-01}
  }

  @online{scw-iam-policy,
    title = {Resource: scaleway_iam_policy},
    author = {{Scaleway}},
    year = {2026},
    url = {https://github.com/scaleway/terraform-provider-scaleway/blob/main/docs/resources/iam_policy.md},
    urldate = {2026-04-01}
  }

  @online{scw-iam-api-key,
    title = {Resource: scaleway_iam_api_key},
    author = {{Scaleway}},
    year = {2026},
    url = {https://github.com/scaleway/terraform-provider-scaleway/blob/main/docs/resources/iam_api_key.md},
    urldate = {2026-04-01}
  }

  @online{scw-object-bucket,
    title = {Resource: scaleway_object_bucket},
    author = {{Scaleway}},
    year = {2026},
    url = {https://github.com/scaleway/terraform-provider-scaleway/blob/main/docs/resources/object_bucket.md},
    urldate = {2026-04-01}
  }

  @online{scw-object-bucket-policy,
    title = {Resource: scaleway_object_bucket_policy},
    author = {{Scaleway}},
    year = {2026},
    url = {https://github.com/scaleway/terraform-provider-scaleway/blob/main/docs/resources/object_bucket_policy.md},
    urldate = {2026-04-01}
  }

  @online{scw-object-bucket-sse,
    title = {Resource: scaleway_object_bucket_server_side_encryption_configuration},
    author = {{Scaleway}},
    year = {2026},
    url = {https://github.com/scaleway/terraform-provider-scaleway/blob/main/docs/resources/object_bucket_server_side_encryption_configuration.md},
    urldate = {2026-04-01}
  }

  @online{scw-object,
    title = {Resource: scaleway_object},
    author = {{Scaleway}},
    year = {2026},
    url = {https://github.com/scaleway/terraform-provider-scaleway/blob/main/docs/resources/object.md},
    urldate = {2026-04-01}
  }

  @online{scw-api-key-object-storage,
    title = {Using IAM API keys with Object Storage},
    author = {{Scaleway}},
    year = {2025},
    note = {Reviewed on July 09, 2025},
    url = {https://www.scaleway.com/en/docs/iam/api-cli/using-api-key-object-storage/},
    urldate = {2026-04-01}
  }

  @online{scw-permission-sets,
    title = {Permission sets},
    author = {{Scaleway}},
    year = {2025},
    url = {https://www.scaleway.com/en/docs/iam/reference-content/permission-sets/},
    urldate = {2026-04-01}
  }

  @online{pulumi-languages,
    title = {Pulumi Languages & SDKs},
    author = {{Pulumi}},
    year = {2026},
    url = {https://www.pulumi.com/docs/iac/languages-sdks/},
    urldate = {2026-04-01}
  }

  @online{pulumi-scaleway,
    title = {Scaleway | Pulumi Registry},
    author = {{Pulumiverse}},
    year = {2026},
    note = {Published on March 09, 2026},
    url = {https://www.pulumi.com/registry/packages/scaleway/},
    urldate = {2026-04-01}
  }

  @online{pulumi-rust-workaround,
    title = {Extending Pulumi's Language Support via YAML},
    author = {{Pulumi}},
    year = {2022},
    note = {Published on June 08, 2022},
    url = {https://www.pulumi.com/blog/extending-pulumi-languages-with-yaml-cue-jsonnet-rust/},
    urldate = {2026-04-01}
  }
  ```.text
)

= #strong[Infrast]ructure #strong[requir]ements

== #strong[Sco]pe #strong[an]d #strong[inp]uts

#strong[Th]is #strong[docu]ment #strong[tur]ns #strong[th]e TASK-2 #strong[rese]arch #strong[in]to #strong[a]n infrastructure-oriented #strong[recomme]ndation #strong[fo]r #strong[ho]w #strong[t]o #strong[declar]atively #strong[dep]loy #strong[an]d #strong[operat]e #strong[th]e #strong[fir]st #strong[itera]tion #strong[o]f #strong[th]e Scaleway-backed #strong[Ni]x #strong[bin]ary cache. #strong[I]t #strong[assume]s #strong[th]e TASK-2 #strong[concl]usions #strong[rem]ain #strong[i]n force: #strong[on]e #strong[privat]e #strong[buc]ket #strong[pe]r #strong[cac]he #strong[envir]onment, #strong[privat]e #strong[rea]ds #strong[an]d #strong[privat]e #strong[wri]tes, #strong[sepa]rate `admin`, `author`, #strong[an]d `consumer` #strong[princ]ipals, a #strong[dedic]ated #strong[Scal]eway #strong[Projec]t, #strong[stan]dard #strong[Ni]x #strong[cac]he #strong[signin]g, #strong[an]d #strong[dir]ect `nix copy` #strong[publi]shing #strong[a]s #strong[th]e #strong[fir]st #strong[impleme]ntation path.

#strong[Th]e #strong[go]al #strong[he]re #strong[i]s #strong[narr]ower #strong[th]an a #strong[fu]ll implementation. #strong[Th]e #strong[docu]ment #strong[ident]ifies #strong[th]e best-fit #strong[decla]rative #strong[toolin]g, #strong[th]e #strong[bound]aries #strong[whe]re #strong[decla]rative #strong[provis]ioning #strong[sto]ps #strong[bei]ng #strong[eno]ugh, #strong[an]d #strong[th]e #strong[sec]ret #strong[mate]rial #strong[th]at #strong[mu]st #strong[exi]st #strong[bef]ore #strong[th]e #strong[cac]he #strong[ca]n #strong[b]e #strong[us]ed safely. TASK-3 #strong[remain]s documentation-only: #strong[i]t #strong[do]es #strong[no]t #strong[provi]sion #strong[infrast]ructure, #strong[gene]rate #strong[crede]ntials, #strong[o]r #strong[acti]vate a cache. @opentofu-faq @opentofu-sensitive-state

== #strong[Evalu]ation #strong[crit]eria

#strong[Th]e #strong[toolin]g #strong[compa]rison #strong[us]es #strong[si]x criteria:

- `Declarative coverage`: #strong[ca]n #strong[th]e #strong[to]ol #strong[provi]sion #strong[th]e #strong[requ]ired #strong[Scal]eway #strong[IA]M #strong[an]d #strong[Obj]ect #strong[Storag]e #strong[resou]rces #strong[dire]ctly?
- `Secret-handling impact`: #strong[ho]w #strong[mu]ch #strong[sec]ret #strong[mate]rial #strong[i]s #strong[lik]ely #strong[t]o #strong[le]ak #strong[in]to #strong[sta]te, #strong[sta]ck #strong[meta]data, #strong[o]r #strong[a]d #strong[ho]c #strong[loc]al #strong[fil]es?
- `State-management risk`: #strong[ho]w #strong[mu]ch #strong[opera]tional #strong[ca]re #strong[i]s #strong[nee]ded #strong[t]o #strong[ke]ep #strong[th]e #strong[too]l's #strong[sta]te #strong[trust]worthy #strong[an]d #strong[recov]erable?
- `Operational simplicity`: #strong[ho]w #strong[ma]ny #strong[ext]ra #strong[runt]imes, #strong[servic]e #strong[depend]encies, #strong[an]d #strong[bespok]e #strong[conve]ntions #strong[do]es #strong[th]e #strong[to]ol #strong[imp]ose?
- `Openness and licensing`: #strong[do]es #strong[th]e #strong[to]ol #strong[fi]t #strong[th]e #strong[proje]ct's #strong[prefe]rence #strong[fo]r #strong[op]en #strong[toolin]g #strong[wi]th a #strong[sta]ble #strong[ecosy]stem?
- `Fit for future automation`: #strong[do]es #strong[th]e #strong[to]ol #strong[lea]ve a #strong[cle]an #strong[boun]dary #strong[fo]r a #strong[lat]er #strong[boots]trap, #strong[publis]h, #strong[an]d #strong[verifi]cation #strong[appli]cation?

#strong[The]se #strong[crit]eria #strong[fav]or #strong[bor]ing, #strong[inspe]ctable #strong[infrast]ructure #strong[manag]ement #strong[ov]er novelty. #strong[Th]e #strong[cac]he #strong[archit]ecture #strong[i]s #strong[sti]ll #strong[sim]ple #strong[eno]ugh #strong[th]at #strong[th]e #strong[be]st #strong[fir]st #strong[to]ol #strong[i]s #strong[th]e #strong[on]e #strong[th]at #strong[expose]s #strong[clo]ud #strong[resou]rces #strong[clearl]y #strong[an]d #strong[lea]ves #strong[opera]tional #strong[gl]ue #strong[i]n a #strong[separ]ately #strong[test]able layer. @opentofu-faq @pulumi-languages

== #strong[Toolin]g #strong[compa]rison

=== #strong[Open]Tofu + #strong[Scal]eway #strong[prov]ider

#strong[Open]Tofu #strong[i]s #strong[th]e #strong[be]st #strong[fi]t #strong[fo]r #strong[th]e #strong[fir]st implementation. #strong[Open]Tofu #strong[expli]citly #strong[wor]ks #strong[wi]th #strong[th]e #strong[curren]t #strong[Terra]form #strong[prov]ider #strong[ecosy]stem, #strong[whi]ch #strong[mea]ns #strong[th]e #strong[exis]ting #strong[Scal]eway #strong[prov]ider #strong[docume]ntation #strong[an]d #strong[reso]urce #strong[mod]el #strong[car]ry #strong[ov]er #strong[withou]t #strong[requi]ring #strong[th]is #strong[repos]itory #strong[t]o #strong[stand]ardize #strong[o]n #strong[Terra]form itself. @opentofu-faq

Pros:

- #strong[I]t #strong[matche]s #strong[th]e #strong[ta]sk well: #strong[th]e #strong[Scal]eway #strong[prov]ider #strong[alread]y #strong[expose]s #strong[IA]M #strong[applic]ations, #strong[IA]M #strong[poli]cies, #strong[IA]M #strong[AP]I #strong[ke]ys, #strong[obj]ect #strong[bucket]s, #strong[buc]ket #strong[poli]cies, server-side #strong[encry]ption #strong[config]uration, #strong[an]d #strong[indiv]idual objects. @scw-iam-application @scw-iam-policy @scw-iam-api-key @scw-object-bucket @scw-object-bucket-policy @scw-object-bucket-sse @scw-object
- #strong[I]t #strong[prese]rves #strong[decla]rative #strong[dri]ft #strong[detec]tion #strong[an]d #strong[reconci]liation #strong[fo]r #strong[clo]ud #strong[resou]rces #strong[instea]d #strong[o]f #strong[mov]ing #strong[th]at #strong[log]ic #strong[in]to a #strong[cus]tom program. @opentofu-faq
- #strong[It]s #strong[sta]te #strong[mod]el #strong[i]s #strong[we]ll #strong[under]stood, #strong[an]d #strong[th]e #strong[docume]ntation #strong[i]s #strong[expl]icit #strong[th]at #strong[sta]te #strong[ma]y #strong[contai]n #strong[sensi]tive #strong[da]ta #strong[an]d #strong[mu]st #strong[b]e #strong[treate]d accordingly. @opentofu-sensitive-state
- #strong[I]t #strong[lea]ves a #strong[cle]an #strong[se]am #strong[fo]r a #strong[sepa]rate #strong[Ru]st #strong[to]ol #strong[t]o #strong[han]dle #strong[opera]tions #strong[th]at #strong[ar]e #strong[proce]dural #strong[rat]her #strong[th]an #strong[decla]rative, #strong[su]ch #strong[a]s #strong[gener]ating #strong[signin]g #strong[ke]ys #strong[o]r #strong[perfo]rming end-to-end #strong[publis]h checks. @nix-generate-key @nix-s3-store

Cons:

- #strong[I]f #strong[th]e #strong[config]uration #strong[create]s #strong[IA]M #strong[AP]I #strong[ke]ys #strong[dire]ctly, #strong[th]e #strong[retu]rned `secret_key` #strong[become]s #strong[sta]te #strong[mate]rial #strong[an]d #strong[mu]st #strong[b]e #strong[prote]cted #strong[li]ke #strong[an]y #strong[oth]er credential. @scw-iam-api-key @opentofu-sensitive-state
- #strong[OpenT]ofu's S3 #strong[backen]d #strong[guid]ance #strong[assume]s AWS-style #strong[lockin]g #strong[vi]a #strong[Dyna]moDB, #strong[whi]ch #strong[do]es #strong[no]t #strong[ma]p #strong[dire]ctly #strong[on]to #strong[Scal]eway #strong[Obj]ect #strong[Storag]e, #strong[s]o a remote-state #strong[stra]tegy #strong[mu]st #strong[b]e #strong[cho]sen #strong[delibe]rately #strong[instea]d #strong[o]f #strong[cop]ied #strong[fr]om #strong[AW]S examples. @opentofu-s3-backend
- #strong[Open]Tofu #strong[ca]n #strong[provi]sion #strong[infrast]ructure, #strong[bu]t #strong[i]t #strong[can]not #strong[b]y #strong[its]elf #strong[expres]s #strong[th]e #strong[who]le #strong[cac]he lifecycle: #strong[Ni]x #strong[signin]g, #strong[publi]cation, #strong[an]d #strong[verifi]cation #strong[sti]ll #strong[si]t #strong[outsid]e #strong[decla]rative #strong[clo]ud resources. @nix-conf @nix-generate-key

=== #strong[Pul]umi + #strong[Scal]eway #strong[prov]ider

#strong[Pul]umi #strong[i]s #strong[via]ble, #strong[bu]t #strong[i]t #strong[i]s #strong[no]t #strong[th]e #strong[stron]gest #strong[recomme]ndation #strong[fo]r #strong[th]is repository. #strong[Th]e #strong[curren]t #strong[Scal]eway #strong[packag]e #strong[i]s #strong[publi]shed #strong[b]y `pulumiverse`, #strong[no]t #strong[b]y #strong[Pul]umi #strong[o]r #strong[Scal]eway #strong[dire]ctly, #strong[an]d #strong[there]fore #strong[do]es #strong[no]t #strong[improv]e #strong[th]e #strong[tru]st #strong[o]r #strong[cove]rage #strong[pictur]e #strong[comp]ared #strong[t]o #strong[usi]ng #strong[Open]Tofu #strong[wi]th #strong[th]e #strong[upst]ream #strong[Scal]eway #strong[prov]ider #strong[do]cs directly. @pulumi-scaleway

Pros:

- #strong[Pul]umi #strong[i]s a #strong[mat]ure #strong[Ia]C #strong[plat]form #strong[wi]th first-class #strong[suppor]t #strong[fo]r #strong[TypeS]cript, #strong[Pyt]hon, #strong[G]o, .NET, #strong[Ja]va, #strong[an]d YAML. @pulumi-languages
- #strong[It]s #strong[Scal]eway #strong[packag]e #strong[expose]s #strong[th]e #strong[sa]me #strong[genera]l #strong[id]ea #strong[a]s #strong[oth]er #strong[Pul]umi providers: a #strong[lang]uage #strong[SD]K #strong[ov]er #strong[decla]rative #strong[reso]urce operations. @pulumi-scaleway
- #strong[I]t #strong[ca]n #strong[b]e #strong[attra]ctive #strong[i]f #strong[th]e #strong[projec]t #strong[lat]er #strong[wan]ts application-like #strong[abstra]ctions #strong[ins]ide #strong[th]e #strong[provis]ioning #strong[lay]er itself. @pulumi-languages

Cons:

- #strong[Th]e #strong[curre]ntly #strong[publi]shed #strong[Scal]eway #strong[packag]e #strong[i]s community-maintained, #strong[whi]ch #strong[i]s a #strong[wea]ker #strong[defaul]t #strong[fo]r a #strong[sma]ll, security-sensitive #strong[cac]he #strong[deplo]yment #strong[th]an #strong[usi]ng #strong[Open]Tofu #strong[agains]t #strong[th]e #strong[mains]tream #strong[Scal]eway #strong[prov]ider documentation. @pulumi-scaleway
- #strong[Pulu]mi's #strong[suppo]rted #strong[lang]uage #strong[li]st #strong[do]es #strong[no]t #strong[includ]e Rust. #strong[Th]e #strong[Pul]umi #strong[te]am #strong[ha]s #strong[sho]wn a Rust-based #strong[YA]ML #strong[gener]ation #strong[worka]round, #strong[bu]t #strong[th]at #strong[i]s #strong[no]t a first-class #strong[Pul]umi #strong[Ru]st #strong[SD]K #strong[an]d #strong[sho]uld #strong[no]t #strong[b]e #strong[treate]d #strong[a]s #strong[ordi]nary #strong[lang]uage support. @pulumi-languages @pulumi-rust-workaround
- #strong[Pul]umi #strong[do]es #strong[no]t #strong[rem]ove #strong[th]e #strong[sa]me #strong[ha]rd #strong[prob]lems #strong[aro]und #strong[signin]g #strong[ke]ys, #strong[crede]ntial #strong[captur]e, #strong[o]r #strong[Ni]x publication. #strong[Tho]se #strong[sti]ll #strong[requir]e #strong[addit]ional glue. @nix-conf @nix-generate-key

=== #strong[Ru]st #strong[an]d #strong[SD]Ks #strong[on]ly

A #strong[pu]re #strong[Ru]st #strong[impleme]ntation #strong[i]s #strong[th]e #strong[wro]ng #strong[defaul]t #strong[fo]r #strong[ba]se provisioning.

Pros:

- #strong[I]t #strong[off]ers #strong[fu]ll #strong[contro]l #strong[ov]er #strong[boots]trap, #strong[publi]cation, #strong[verifi]cation, #strong[an]d #strong[sec]ret #strong[handof]f behavior.
- #strong[I]t #strong[wou]ld #strong[ke]ep #strong[th]e #strong[opera]tional #strong[gl]ue #strong[i]n #strong[th]e #strong[sa]me #strong[impleme]ntation #strong[lang]uage #strong[th]at #strong[th]e #strong[projec]t #strong[alread]y #strong[prefer]s #strong[fo]r non-trivial automation.

Cons:

- #strong[I]t #strong[wou]ld #strong[replac]e #strong[decla]rative #strong[reconci]liation, #strong[imp]ort, #strong[an]d #strong[dri]ft #strong[inspe]ction #strong[wi]th #strong[cus]tom #strong[imper]ative logic.
- #strong[I]t #strong[wou]ld #strong[for]ce #strong[th]is #strong[repos]itory #strong[t]o #strong[ow]n #strong[prov]ider #strong[cove]rage, #strong[err]or #strong[mode]ling, #strong[retrie]s, #strong[an]d #strong[sta]te #strong[seman]tics #strong[th]at #strong[Open]Tofu #strong[alread]y #strong[sol]ves #strong[we]ll enough.
- #strong[I]t #strong[wou]ld #strong[bl]ur #strong[tw]o #strong[diffe]rent concerns: #strong[clo]ud #strong[reso]urce #strong[decla]ration #strong[ver]sus #strong[cac]he #strong[work]flow automation.

=== #strong[Be]st #strong[curren]t #strong[recomme]ndation

#strong[Us]e `OpenTofu + the Scaleway provider` #strong[fo]r #strong[infrast]ructure provisioning. #strong[Ke]ep #strong[Pul]umi #strong[a]s a #strong[via]ble #strong[bu]t #strong[secon]dary #strong[opt]ion, #strong[an]d #strong[us]e #strong[Ru]st #strong[on]ly #strong[fo]r #strong[th]e #strong[proce]dural #strong[gl]ue #strong[lay]er #strong[th]at #strong[Open]Tofu #strong[sho]uld #strong[no]t own. #strong[Th]at #strong[spl]it #strong[kee]ps #strong[reso]urce #strong[int]ent #strong[decla]rative #strong[whi]le #strong[giv]ing #strong[th]e #strong[fut]ure #strong[work]flow #strong[co]de a #strong[cle]ar, #strong[nar]row #strong[respons]ibility set. @opentofu-faq @pulumi-scaleway

== #strong[Decla]rative #strong[cove]rage #strong[mat]rix

#table(
  columns: (1.5fr, 1.4fr, 1fr, 1.6fr),
  table.header([Capability], [OpenTofu coverage], [Should be declarative?], [Important caveat]),
  [IAM applications and IAM policies],
  [Direct via `scaleway_iam_application` and `scaleway_iam_policy`. @scw-iam-application @scw-iam-policy],
  [Yes],
  [This is the cleanest place to encode role separation and Project scoping. @scw-permission-sets],
  [IAM API keys],
  [Direct via `scaleway_iam_api_key`. @scw-iam-api-key],
  [Sometimes],
  [The returned `secret_key` becomes state material. Only do this if the state backend is treated as sensitive infrastructure. @scw-iam-api-key @opentofu-sensitive-state],
  [Object bucket],
  [Direct via `scaleway_object_bucket`, including region and lifecycle rules. @scw-object-bucket],
  [Yes],
  [Versioning and lifecycle should be chosen deliberately because cache retention mistakes are expensive. @scw-object-bucket],
  [Bucket policy],
  [Direct via `scaleway_object_bucket_policy`. @scw-object-bucket-policy],
  [Yes],
  [Project scoping matters for child S3 resources; wrong `project_id` selection can produce `403` behavior. @scw-object-bucket-policy @scw-api-key-object-storage],
  [Server-side encryption defaults],
  [Direct via `scaleway_object_bucket_server_side_encryption_configuration`. @scw-object-bucket-sse],
  [Yes],
  [This covers bucket-side encryption defaults, not Nix signing. @scw-object-bucket-sse @nix-conf],
  [Seed objects or static markers],
  [Direct via `scaleway_object`. @scw-object],
  [Optional],
  [Useful for markers or bootstrap metadata, but not for secret signing material. @scw-object @nix-conf]
)

#strong[Th]e #strong[concl]usion #strong[fr]om #strong[th]e #strong[mat]rix #strong[i]s straightforward: #strong[Open]Tofu #strong[cov]ers #strong[th]e #strong[requ]ired #strong[Scal]eway #strong[infrast]ructure #strong[surfac]e adequately. #strong[Th]e #strong[ma]in #strong[rea]son #strong[t]o #strong[st]op #strong[sho]rt #strong[o]f "#strong[every]thing #strong[i]n #strong[Open]Tofu" #strong[i]s #strong[no]t #strong[missin]g #strong[prov]ider #strong[cove]rage, #strong[bu]t #strong[th]e #strong[ri]sk #strong[o]f #strong[storin]g #strong[new]ly #strong[gener]ated #strong[crede]ntials #strong[i]n #strong[to]ol #strong[sta]te #strong[an]d #strong[th]e #strong[mism]atch #strong[betwee]n #strong[decla]rative #strong[Ia]C #strong[an]d #strong[opera]tional #strong[Ni]x workflows. @opentofu-sensitive-state @scw-iam-api-key

== #strong[Wh]at #strong[Open]Tofu #strong[can]not #strong[saf]ely #strong[o]r #strong[cleanl]y #strong[d]o #strong[alo]ne

#strong[Open]Tofu #strong[sho]uld #strong[no]t #strong[b]e #strong[stret]ched #strong[in]to #strong[own]ing #strong[th]e #strong[who]le #strong[cac]he workflow. #strong[Th]e #strong[follo]wing #strong[ste]ps #strong[ar]e #strong[proce]dural #strong[an]d #strong[sho]uld #strong[instea]d #strong[b]e #strong[assi]gned #strong[t]o a #strong[fut]ure #strong[Ru]st #strong[CL]I #strong[o]r #strong[sma]ll #strong[Ru]st application:

- `Signing key bootstrap`: #strong[gene]rate #strong[th]e #strong[bin]ary #strong[cac]he #strong[signin]g #strong[keypai]r #strong[wi]th `nix-store --generate-binary-cache-key`, #strong[ke]ep #strong[th]e #strong[privat]e #strong[ke]y #strong[ou]t #strong[o]f #strong[obj]ect #strong[storag]e, #strong[an]d #strong[publis]h #strong[on]ly #strong[th]e #strong[pub]lic #strong[ke]y #strong[t]o consumers. @nix-generate-key @nix-conf
- `Credential capture and escrow`: #strong[wh]en #strong[a]n #strong[AP]I #strong[ke]y #strong[o]r #strong[oth]er one-time #strong[sec]ret #strong[i]s #strong[create]d, #strong[captur]e #strong[i]t #strong[on]ce #strong[an]d #strong[immed]iately #strong[pla]ce #strong[i]t #strong[in]to `sops`-encrypted #strong[storag]e #strong[fo]r #strong[th]e #strong[confi]gured #strong[recip]ients #strong[und]er #strong[th]is #strong[subpro]ject's policy. @scw-iam-api-key @opentofu-sensitive-state
- `Publish workflow`: #strong[ru]n `nix copy` #strong[wi]th #strong[th]e #strong[correc]t `s3://` #strong[endp]oint, #strong[reg]ion, #strong[an]d #strong[signin]g #strong[ke]y #strong[inp]uts #strong[s]o #strong[publi]shed #strong[object]s #strong[ar]e #strong[bo]th #strong[uplo]aded #strong[an]d #strong[sig]ned #strong[i]n #strong[th]e #strong[wa]y #strong[consu]mers expect. @nix-s3-store @nix-conf
- `Verification workflow`: #strong[pro]ve #strong[th]at #strong[th]e `author` #strong[iden]tity #strong[ca]n #strong[wri]te, #strong[th]e `consumer` #strong[iden]tity #strong[ca]n #strong[re]ad, #strong[an]d #strong[consu]mers #strong[tru]st #strong[th]e #strong[cac]he #strong[signa]tures #strong[prod]uced #strong[b]y #strong[th]e #strong[confi]gured #strong[signin]g key. @nix-s3-store @nix-conf

#strong[Th]is #strong[divi]sion #strong[i]s intentional. #strong[Ru]st #strong[i]s #strong[no]t #strong[bei]ng #strong[prop]osed #strong[a]s a #strong[repla]cement #strong[fo]r #strong[Ia]C; #strong[i]t #strong[i]s #strong[th]e #strong[gl]ue #strong[lay]er #strong[fo]r #strong[boots]trap, #strong[publis]h, #strong[an]d #strong[ver]ify #strong[tas]ks #strong[th]at #strong[ar]e #strong[awkwar]d, #strong[stat]eful, #strong[o]r one-shot #strong[i]n #strong[wa]ys #strong[decla]rative #strong[too]ls #strong[han]dle poorly.

== #strong[Secret]s #strong[an]d #strong[crede]ntial #strong[inven]tory

#table(
  columns: (1.4fr, 1fr, 0.8fr, 1.3fr, 1.2fr),
  table.header([Credential or value], [Used by], [Secret?], [Storage expectation], [Provisioned by]),
  [OpenTofu operator/admin Scaleway credentials],
  [`admin` operator or CI],
  [Yes],
  [Operator environment, external secret store, or `sops` if repo-managed bootstrap requires it],
  [Human operator or platform-level secret system],
  [Author API access key],
  [`author` automation],
  [Yes],
  [Encrypted with `sops` after creation; never committed in plaintext],
  [OpenTofu or manual bootstrap, then captured by Rust workflow],
  [Author API secret key],
  [`author` automation],
  [Yes],
  [Encrypted with `sops` after creation; treat as one-time capture material],
  [OpenTofu or manual bootstrap, then captured by Rust workflow],
  [Consumer API access key],
  [`consumer` systems],
  [Yes],
  [Encrypted with `sops` or provisioned through the consumer environment's own secret system],
  [OpenTofu or manual bootstrap],
  [Consumer API secret key],
  [`consumer` systems],
  [Yes],
  [Encrypted with `sops` or provisioned through the consumer environment's own secret system],
  [OpenTofu or manual bootstrap],
  [Nix binary cache private signing key],
  [`author` publish workflow],
  [Yes],
  [Encrypted with `sops`; never stored in the bucket],
  [Rust bootstrap workflow using `nix-store --generate-binary-cache-key`],
  [Nix binary cache public key],
  [`consumer` configuration],
  [No],
  [Tracked as configuration or distributed in system config],
  [Derived from the signing keypair],
  [Optional remote-state backend credentials],
  [OpenTofu runtime],
  [Yes],
  [Separate from cache credentials; stored wherever the chosen state backend expects],
  [Human operator or platform-level secret system]
)

#strong[Th]e #strong[follo]wing #strong[val]ues #strong[ar]e #strong[impor]tant #strong[config]uration #strong[bu]t #strong[ar]e #strong[no]t #strong[thems]elves secrets: #strong[buc]ket #strong[na]me, #strong[Scal]eway #strong[reg]ion, S3 #strong[endp]oint, #strong[Projec]t #strong[I]D, #strong[IA]M #strong[appli]cation #strong[ID]s, #strong[buc]ket #strong[pol]icy #strong[JS]ON, #strong[an]d #strong[th]e #strong[pub]lic #strong[signin]g key. #strong[Th]ey #strong[ma]y #strong[sti]ll #strong[deserv]e #strong[cha]nge #strong[contro]l, #strong[bu]t #strong[th]ey #strong[d]o #strong[no]t #strong[requir]e #strong[th]e #strong[sa]me #strong[hand]ling #strong[a]s credentials. @scw-api-key-object-storage @nix-conf

== #strong[Inter]face #strong[boun]dary #strong[fo]r #strong[lat]er #strong[impleme]ntation

TASK-3 #strong[ad]ds #strong[n]o #strong[runtim]e #strong[AP]I, #strong[bu]t #strong[i]t #strong[do]es #strong[def]ine #strong[th]e #strong[inte]nded #strong[owner]ship #strong[boun]dary #strong[fo]r #strong[fut]ure code:

- `OpenTofu` #strong[ow]ns #strong[clo]ud #strong[reso]urce #strong[declar]ations, drift-managed #strong[infrast]ructure, #strong[an]d non-secret #strong[output]s #strong[su]ch #strong[a]s #strong[buc]ket #strong[nam]es, #strong[region]s, #strong[endpo]ints, #strong[pol]icy #strong[attac]hments, #strong[an]d #strong[appli]cation identifiers.
- `Rust CLI/application` #strong[ow]ns `bootstrap`, `publish`, #strong[an]d `verify` workflows.
- `sops`-encrypted #strong[fil]es #strong[und]er `cache/scaleway` #strong[rem]ain #strong[th]e repository-local #strong[sec]ret #strong[storag]e #strong[mecha]nism #strong[wh]en #strong[secret]s #strong[mu]st #strong[li]ve #strong[along]side #strong[th]is subproject.

#strong[Th]at #strong[boun]dary #strong[kee]ps #strong[th]e #strong[cac]he #strong[contro]l #strong[pla]ne legible. #strong[Open]Tofu #strong[descr]ibes #strong[wh]at #strong[sho]uld #strong[exi]st #strong[i]n #strong[Scal]eway; #strong[th]e #strong[Ru]st #strong[toolin]g #strong[perf]orms #strong[th]e #strong[sma]ll #strong[num]ber #strong[o]f #strong[action]s #strong[th]at #strong[mu]st #strong[hap]pen #strong[i]n #strong[sequ]ence #strong[an]d #strong[wi]th #strong[expl]icit #strong[sec]ret handling.

== #strong[Recom]mended #strong[impleme]ntation #strong[appro]aches

=== #strong[Appr]oach A: #strong[Open]Tofu #strong[fo]r #strong[infrast]ructure, #strong[Ru]st #strong[fo]r #strong[opera]tional #strong[gl]ue

#strong[Th]is #strong[i]s #strong[th]e #strong[recom]mended approach.

Flow:

1. #strong[Open]Tofu #strong[provi]sions #strong[th]e #strong[dedic]ated Project-scoped #strong[ident]ities, #strong[poli]cies, #strong[buc]ket, #strong[buc]ket #strong[pol]icy, #strong[an]d bucket-side #strong[encry]ption defaults.
2. A #strong[Ru]st #strong[boots]trap #strong[comman]d #strong[gener]ates #strong[th]e #strong[Ni]x #strong[signin]g #strong[keypai]r #strong[an]d, #strong[i]f #strong[neces]sary, #strong[capt]ures #strong[new]ly #strong[create]d #strong[AP]I #strong[ke]y #strong[mate]rial #strong[exactl]y once.
3. #strong[Th]e #strong[Ru]st #strong[boots]trap #strong[comman]d #strong[wri]tes #strong[sec]ret #strong[output]s #strong[in]to `sops`-encrypted #strong[fil]es #strong[fo]r #strong[th]e #strong[confi]gured recipients.
4. A #strong[Ru]st #strong[publis]h #strong[comman]d #strong[ru]ns `nix copy` #strong[wi]th #strong[th]e #strong[correc]t #strong[endp]oint, #strong[reg]ion, #strong[an]d `secret-key-files` inputs.
5. A #strong[Ru]st #strong[ver]ify #strong[comman]d #strong[che]cks #strong[aut]hor #strong[wri]te #strong[acc]ess, #strong[cons]umer #strong[re]ad #strong[acc]ess, #strong[an]d #strong[signa]ture #strong[tru]st #strong[en]d #strong[t]o end.

#strong[Wh]y #strong[i]t #strong[i]s preferred:

- #strong[Infrast]ructure #strong[sta]ys #strong[decla]rative #strong[an]d reviewable.
- #strong[Sec]ret #strong[captur]e #strong[an]d #strong[Ni]x #strong[signin]g #strong[st]ay #strong[outsid]e #strong[th]e #strong[Ia]C #strong[sta]te path.
- #strong[Th]e #strong[Ru]st #strong[lay]er #strong[i]s #strong[sma]ll, #strong[test]able, #strong[an]d #strong[operat]ionally #strong[focuse]d #strong[rat]her #strong[th]an #strong[beco]ming a #strong[sha]dow #strong[provis]ioning system.

#strong[Sta]te requirement:

- #strong[Open]Tofu #strong[sta]te #strong[mu]st #strong[sti]ll #strong[b]e #strong[treate]d #strong[a]s #strong[sensi]tive #strong[an]d #strong[mu]st #strong[li]ve #strong[separ]ately #strong[fr]om #strong[th]e #strong[bin]ary #strong[cac]he bucket. #strong[I]f #strong[rem]ote #strong[sta]te #strong[i]s #strong[us]ed, #strong[it]s #strong[buc]ket #strong[an]d #strong[crede]ntials #strong[sho]uld #strong[b]e #strong[isol]ated #strong[fr]om #strong[th]e #strong[cache']s #strong[rea]der #strong[an]d #strong[wri]ter credentials. @opentofu-sensitive-state @opentofu-s3-backend

=== #strong[Appr]oach B: #strong[Open]Tofu #strong[provi]sions #strong[infrast]ructure #strong[an]d #strong[AP]I #strong[ke]ys, #strong[Ru]st #strong[handle]s #strong[signin]g #strong[an]d #strong[publi]cation

#strong[Th]is #strong[i]s #strong[accep]table, #strong[bu]t weaker.

Flow:

1. #strong[Open]Tofu #strong[provi]sions #strong[th]e #strong[sa]me #strong[infrast]ructure #strong[a]s #strong[i]n #strong[Appr]oach A #strong[an]d #strong[al]so #strong[create]s `author` #strong[an]d `consumer` #strong[AP]I keys.
2. #strong[Th]e #strong[resul]ting #strong[ke]y #strong[mate]rial #strong[i]s #strong[expo]rted #strong[fr]om #strong[Open]Tofu #strong[output]s #strong[o]r #strong[other]wise #strong[capt]ured immediately.
3. A #strong[Ru]st #strong[boots]trap #strong[comman]d #strong[mov]es #strong[tho]se #strong[crede]ntials #strong[in]to `sops`-encrypted #strong[storag]e #strong[an]d #strong[gener]ates #strong[th]e #strong[Ni]x #strong[signin]g keypair.
4. #strong[Ru]st #strong[sti]ll #strong[ow]ns #strong[publi]cation #strong[an]d verification.

#strong[Wh]y #strong[i]t #strong[i]s weaker:

- #strong[Th]e `secret_key` #strong[val]ues #strong[fo]r #strong[IA]M #strong[AP]I #strong[ke]ys #strong[ar]e #strong[no]w #strong[presen]t #strong[i]n #strong[Open]Tofu #strong[sta]te, #strong[whi]ch #strong[rai]ses #strong[th]e #strong[sensi]tivity #strong[o]f #strong[th]e #strong[sta]te #strong[backen]d materially. @scw-iam-api-key @opentofu-sensitive-state
- #strong[Reco]very, #strong[audi]ting, #strong[an]d #strong[acc]ess #strong[rev]iew #strong[fo]r #strong[sta]te #strong[bec]ome #strong[pa]rt #strong[o]f #strong[th]e #strong[crede]ntial #strong[secu]rity story.

#strong[Wh]en #strong[t]o #strong[acc]ept it:

- #strong[Wh]en #strong[redu]cing #strong[man]ual #strong[boots]trap #strong[ste]ps #strong[matter]s #strong[mo]re #strong[th]an #strong[minim]izing #strong[sta]te #strong[expo]sure, #strong[an]d #strong[th]e #strong[te]am #strong[i]s #strong[willin]g #strong[t]o #strong[har]den #strong[an]d #strong[isolat]e #strong[th]e #strong[sta]te #strong[backen]d accordingly.

=== #strong[Fut]ure #strong[opti]onal #strong[evolu]tion

#strong[I]f #strong[lat]er #strong[tas]ks #strong[disc]over #strong[th]at #strong[dir]ect #strong[publi]shing #strong[i]s #strong[operat]ionally #strong[insuff]icient, a #strong[dedic]ated #strong[interm]ediary #strong[o]r #strong[mo]re #strong[opini]onated #strong[servic]e #strong[lay]er #strong[ca]n #strong[b]e #strong[add]ed later. #strong[Th]at #strong[wou]ld #strong[b]e a second-step #strong[archit]ecture #strong[cha]nge, #strong[no]t a #strong[rea]son #strong[t]o #strong[avo]id #strong[th]e #strong[simple]r OpenTofu-plus-Rust #strong[spl]it today.

== #strong[Curren]t #strong[recomme]ndation

- #strong[Us]e #strong[Open]Tofu #strong[a]s #strong[th]e #strong[primar]y #strong[provis]ioning #strong[to]ol #strong[fo]r #strong[th]e #strong[Scal]eway #strong[cac]he infrastructure. @opentofu-faq
- #strong[Ke]ep #strong[Pul]umi #strong[ou]t #strong[o]f #strong[th]e #strong[fir]st #strong[impleme]ntation #strong[pa]th; #strong[i]t #strong[i]s #strong[via]ble, #strong[bu]t #strong[curre]ntly #strong[wea]ker #strong[becaus]e #strong[th]e #strong[Scal]eway #strong[packag]e #strong[i]s community-maintained #strong[an]d #strong[Ru]st #strong[i]s #strong[no]t a first-class #strong[Pul]umi language. @pulumi-languages @pulumi-scaleway @pulumi-rust-workaround
- #strong[Reserv]e #strong[Ru]st #strong[fo]r #strong[th]e #strong[thr]ee #strong[proce]dural #strong[workf]lows #strong[th]at #strong[rem]ain #strong[aft]er provisioning: `bootstrap`, `publish`, #strong[an]d `verify`.
- #strong[Tre]at #strong[Open]Tofu #strong[sta]te #strong[a]s #strong[sensi]tive, #strong[espec]ially #strong[i]f #strong[AP]I #strong[ke]ys #strong[ar]e #strong[create]d declaratively. @opentofu-sensitive-state @scw-iam-api-key
- #strong[Ke]ep #strong[th]e #strong[sta]te #strong[backen]d #strong[sepa]rate #strong[fr]om #strong[th]e #strong[cac]he #strong[buc]ket #strong[an]d #strong[fr]om #strong[ordi]nary #strong[cac]he #strong[cons]umer credentials. @opentofu-s3-backend

== #strong[Refer]ences

#bibliography(refs, style: "ieee", title: none)
