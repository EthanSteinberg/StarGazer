$(function(){
    var stars = [];

    function getStars(messageID, callback) {
        return $.ajax({
            type: "GET",
            url: "http://45.55.75.16:80/stars",
            data: {message_id: messageID},
            success: function(value){callback(JSON.parse(value));},
        });
    }

    function parseID(id) {
        return parseInt(id.substring(8));
    }

    function getStarsFromDOM(star, box) {
        box.html(''); // Clear the box out
        var myMessageID = parseID(star.parents('li, .message').attr('id'));
        return getStars(myMessageID, function(stars){
            box.append('<h4> Starred by: </h4>');
            var innerList = $('<ul>');
            var names = stars.map(function(star){
                return star.user_name;
            });
            for (var name of names) {
                var listItem = $('<li>');
                listItem.text(name);
                innerList.append(listItem);
            }
            box.append(innerList);
        });
    }

    function setUpStar(star) {
        var box = $('<div class="blah"/>');
        star.append(box);
        var currentRequest = getStarsFromDOM(star, box);
        star.parent().mouseenter(function(){
            currentRequest.abort();
            currentRequest = getStarsFromDOM(star, box);
            box.show();
        });
        star.parent().mouseleave(function(){
            box.hide();
        });
    }

    function lookForMoreStars() {
        var newStars = Array.from($("span.stars:visible")).filter(function(newStar){
            return $(newStar).parent("span.meta").length == 0;
        }).filter(function(newStar){
            return stars.every(function(star){
                return !star.isSameNode(newStar);
            });
        });

        for (var newStar of newStars) {
            stars.push(newStar);
            setUpStar($(newStar));
        }
    }

    window.setInterval(lookForMoreStars, 1000);
});