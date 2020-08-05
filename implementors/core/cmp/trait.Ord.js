(function() {var implementors = {};
implementors["arrayvec"] = [{"text":"impl&lt;A&gt; Ord for ArrayString&lt;A&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;A: Array&lt;Item = u8&gt; + Copy,&nbsp;</span>","synthetic":false,"types":[]},{"text":"impl&lt;T:&nbsp;Ord&gt; Ord for CapacityError&lt;T&gt;","synthetic":false,"types":[]},{"text":"impl&lt;A:&nbsp;Array&gt; Ord for ArrayVec&lt;A&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;A::Item: Ord,&nbsp;</span>","synthetic":false,"types":[]}];
implementors["byteorder"] = [{"text":"impl Ord for BigEndian","synthetic":false,"types":[]},{"text":"impl Ord for LittleEndian","synthetic":false,"types":[]}];
implementors["lexical_core"] = [{"text":"impl Ord for ErrorCode","synthetic":false,"types":[]},{"text":"impl Ord for Error","synthetic":false,"types":[]}];
implementors["proc_macro2"] = [{"text":"impl Ord for Ident","synthetic":false,"types":[]}];
implementors["syn"] = [{"text":"impl Ord for Lifetime","synthetic":false,"types":[]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()